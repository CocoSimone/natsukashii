#include <Audio.hpp>
#include <SDL2/SDL.h>
#include <util.hpp>

namespace natsukashii::core {
#define AUDIO_SAMPLE_RATE 48000
#define SYSTEM_SAMPLE_FORMAT AUDIO_F32SYS
#define SYSTEM_SAMPLE_SIZE 4
#define BYTES_PER_HALF_SECOND ((AUDIO_SAMPLE_RATE / 2) * SYSTEM_SAMPLE_SIZE)

static SDL_AudioStream* audioStream = nullptr;
SDL_mutex* audioStreamMutex;
SDL_AudioSpec audioSpec;
SDL_AudioSpec request;
SDL_AudioDeviceID audioDev;

#define LockAudioMutex() SDL_LockMutex(audioStreamMutex)
#define UnlockAudioMutex() SDL_UnlockMutex(audioStreamMutex)

void audioCallback(void* userdata, Uint8* stream, int length) {
  int gotten = 0;
  LockAudioMutex();
  int available = SDL_AudioStreamAvailable(audioStream);

  if (available > 0) {
    gotten = SDL_AudioStreamGet(audioStream, stream, length);
  }
  UnlockAudioMutex();

  int gotten_samples = (int)(gotten / sizeof(float));
  auto* out = (float*)stream;
  out += gotten_samples;

  for (int i = gotten_samples; i < length / sizeof(float); i++) {
    float sample = 0;
    *out++ = sample;
  }
}

void InitAudio() {
  AdjustSampleRate(AUDIO_SAMPLE_RATE);
  memset(&request, 0, sizeof(request));

  request.freq = AUDIO_SAMPLE_RATE;
  request.format = SYSTEM_SAMPLE_FORMAT;
  request.channels = 2;
  request.samples = 1024;
  request.callback = audioCallback;
  request.userdata = nullptr;

  audioDev = SDL_OpenAudioDevice(nullptr, 0, &request, &audioSpec, 0);

  if(!audioDev) {
    util::panic("Failed to initialize SDL Audio: {}", SDL_GetError());
  }

  SDL_PauseAudioDevice(audioDev, false);

  audioStreamMutex = SDL_CreateMutex();

  if(!audioStreamMutex) {
    util::panic("Unable to initialize audio mutex: {}", SDL_GetError());
  }
}

void PushSample(s16 left, s16 right) {
  s16 samples[2]{ left, right };

  int availableBytes = SDL_AudioStreamAvailable(audioStream);
  if(availableBytes < BYTES_PER_HALF_SECOND) {
    SDL_AudioStreamPut(audioStream, samples, 2 * sizeof(16));
  }
}

void AdjustSampleRate(int sampleRate) {
  LockAudioMutex();
  if(audioStream) SDL_FreeAudioStream(audioStream);

  audioStream = SDL_NewAudioStream(AUDIO_S16SYS, 2, sampleRate, SYSTEM_SAMPLE_FORMAT, 2, AUDIO_SAMPLE_RATE);
  UnlockAudioMutex();
}

}