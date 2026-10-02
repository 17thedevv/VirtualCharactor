// Audio Player with Web Audio API Analyser and High-Fidelity RVC Anime Voice Synthesis (Exclusive: 🌸 Onee-san Yae Miko)

export type VoiceEngineMode = 'yaemiko';

class CharacterAudioPlayer {
  private audioCtx: AudioContext | null = null;
  private currentBufferSource: AudioBufferSourceNode | null = null;
  private currentGainNode: GainNode | null = null;
  private analyser: AnalyserNode | null = null;
  private animFrameId: number | null = null;
  private isMutedState = false;
  private isPlayingState = false;
  private isLoadingState = false;

  private onAmplitudeCallback?: (amp: number) => void;
  private onPlaybackStateCallback?: (isPlaying: boolean) => void;
  private onLoadingStateCallback?: (isLoading: boolean) => void;

  constructor() {
    const savedMuted = localStorage.getItem('aria_audio_muted');
    if (savedMuted !== null) {
      this.isMutedState = savedMuted === 'true';
    }

    // Always locked to Onee-san Yae Miko
    localStorage.setItem('aria_voice_engine', 'yaemiko');
  }

  public getVoiceEngine(): VoiceEngineMode {
    return 'yaemiko';
  }

  public isMuted(): boolean {
    return this.isMutedState;
  }

  public isPlaying(): boolean {
    return this.isPlayingState;
  }

  public isLoading(): boolean {
    return this.isLoadingState;
  }

  public setMuted(muted: boolean) {
    this.isMutedState = muted;
    localStorage.setItem('aria_audio_muted', String(muted));
    if (muted) {
      this.stop();
    }
  }

  public setVolume(volume: number) {
    if (this.currentGainNode) {
      this.currentGainNode.gain.value = Math.max(0, Math.min(1, volume));
    }
  }

  public setCallbacks(
    onAmplitude: (amp: number) => void,
    onPlaybackState: (isPlaying: boolean) => void,
    onLoadingState?: (isLoading: boolean) => void
  ) {
    this.onAmplitudeCallback = onAmplitude;
    this.onPlaybackStateCallback = onPlaybackState;
    this.onLoadingStateCallback = onLoadingState;
  }

  /**
   * Unlock AudioContext on user interaction
   */
  public async unlockAudioContext(): Promise<void> {
    try {
      if (!this.audioCtx) {
        const AudioContextClass = window.AudioContext || (window as any).webkitAudioContext;
        this.audioCtx = new AudioContextClass();
      }
      if (this.audioCtx.state === 'suspended') {
        await this.audioCtx.resume();
      }
    } catch (err) {
      console.warn('Could not unlock AudioContext:', err);
    }
  }

  /**
   * Primary entry point: Speaks text with Onee-san Yae Miko RVC Voice.
   */
  public async speak(text: string): Promise<void> {
    const cleanText = text
      .replace(/\*[^*]+\*/g, '') // remove action descriptions like *mỉm cười*
      .replace(/[#`_~"]/g, '')
      .trim();

    if (!cleanText || this.isMutedState) return;

    const encoded = encodeURIComponent(cleanText);
    const url = `http://127.0.0.1:3000/api/audio/tts?text=${encoded}&voice=yaemiko`;
    await this.playUrl(url);
  }

  /**
   * Play speech audio from URL with automatic Web Audio decoding and analyser
   */
  public async playUrl(audioUrl: string): Promise<void> {
    if (this.isMutedState) return;

    try {
      this.stop();
      this.isLoadingState = true;
      this.onLoadingStateCallback?.(true);

      await this.unlockAudioContext();

      if (!this.audioCtx) {
        throw new Error('AudioContext not available');
      }

      // Ensure voice=yaemiko is always attached
      let finalUrl = audioUrl;
      if (!finalUrl.includes('voice=')) {
        const sep = finalUrl.includes('?') ? '&' : '?';
        finalUrl = `${finalUrl}${sep}voice=yaemiko`;
      }

      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 12000);

      let res: Response;
      try {
        res = await fetch(finalUrl, { signal: controller.signal });
      } finally {
        clearTimeout(timeoutId);
      }

      if (!res.ok) {
        throw new Error(`TTS server returned status ${res.status}`);
      }

      const arrayBuffer = await res.arrayBuffer();
      if (!arrayBuffer || arrayBuffer.byteLength === 0) {
        throw new Error('Received empty audio buffer from TTS server');
      }

      // Decode MP3 audio into AudioBuffer
      const audioBuffer = await this.audioCtx.decodeAudioData(arrayBuffer);

      this.isLoadingState = false;
      this.onLoadingStateCallback?.(false);

      // Play buffer through Web Audio graph with analyser
      this.playBuffer(audioBuffer);
    } catch (err) {
      console.error('Anime RVC TTS playback error (Yae Miko):', err);
      this.isLoadingState = false;
      this.onLoadingStateCallback?.(false);
      this.stop();
    }
  }

  private playBuffer(audioBuffer: AudioBuffer) {
    if (!this.audioCtx) return;

    const source = this.audioCtx.createBufferSource();
    source.buffer = audioBuffer;

    const analyser = this.audioCtx.createAnalyser();
    analyser.fftSize = 256;

    const gainNode = this.audioCtx.createGain();
    gainNode.gain.value = 1.0;

    source.connect(analyser);
    analyser.connect(gainNode);
    gainNode.connect(this.audioCtx.destination);

    this.currentBufferSource = source;
    this.currentGainNode = gainNode;
    this.analyser = analyser;
    this.isPlayingState = true;

    source.onended = () => {
      this.stop();
    };

    source.start(0);
    this.onPlaybackStateCallback?.(true);
    this.startTracking();
  }

  public stop() {
    if (this.currentBufferSource) {
      try {
        this.currentBufferSource.stop();
      } catch {
        // Source may already be stopped
      }
      this.currentBufferSource = null;
    }

    if (this.isLoadingState) {
      this.isLoadingState = false;
      this.onLoadingStateCallback?.(false);
    }

    this.isPlayingState = false;
    this.stopTracking();
    this.onPlaybackStateCallback?.(false);
  }

  private startTracking() {
    if (this.animFrameId) cancelAnimationFrame(this.animFrameId);

    const update = () => {
      if (this.analyser && this.isPlayingState) {
        const dataArray = new Uint8Array(this.analyser.frequencyBinCount);
        this.analyser.getByteFrequencyData(dataArray);

        let sum = 0;
        for (let i = 0; i < dataArray.length; i++) {
          sum += dataArray[i];
        }
        const avg = sum / dataArray.length;
        const normalized = Math.min(1.0, (avg / 128.0) * 1.6);
        this.onAmplitudeCallback?.(normalized);
        this.animFrameId = requestAnimationFrame(update);
      } else {
        this.onAmplitudeCallback?.(0);
      }
    };

    update();
  }

  private stopTracking() {
    if (this.animFrameId) {
      cancelAnimationFrame(this.animFrameId);
      this.animFrameId = null;
    }
    this.onAmplitudeCallback?.(0);
  }
}

export const audioPlayer = new CharacterAudioPlayer();
