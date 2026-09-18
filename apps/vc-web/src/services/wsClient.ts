export type WsEventListener = (event: string, data: any) => void;

export class VirtualCharacterClient {
  private ws: WebSocket | null = null;
  private url: string;
  private listeners: Set<WsEventListener> = new Set();
  private reconnectTimer: number | null = null;
  private isConnected = false;
  private isSimulated = false;

  constructor(url = 'ws://127.0.0.1:3000/ws/interaction') {
    this.url = url;
  }

  public getIsSimulated(): boolean {
    return this.isSimulated;
  }

  public connect() {
    try {
      this.ws = new WebSocket(this.url);

      this.ws.onopen = () => {
        this.isConnected = true;
        this.isSimulated = false;
        this.notify('connection_change', { status: 'connected' });
      };

      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          if (data.event) {
            this.notify(data.event, data);
          }
        } catch (err) {
          console.error('[VC-WS] Failed to parse message', err);
        }
      };

      this.ws.onclose = () => {
        this.isConnected = false;
        this.notify('connection_change', { status: 'disconnected' });
        this.scheduleReconnect();
      };

      this.ws.onerror = () => {
        // Fallback to simulated local runtime if server is offline
        if (!this.isConnected) {
          this.isSimulated = true;
          this.notify('connection_change', { status: 'simulated' });
        }
      };
    } catch {
      this.isSimulated = true;
      this.notify('connection_change', { status: 'simulated' });
    }
  }

  public subscribe(listener: WsEventListener) {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private notify(event: string, data: any) {
    for (const listener of this.listeners) {
      listener(event, data);
    }
  }

  public sendMessage(text: string, actorId = 'user-default') {
    if (this.isConnected && this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(
        JSON.stringify({
          type: 'message',
          text,
          actor_id: actorId,
        })
      );
    } else {
      // Local client-side simulation when backend is unreachable
      this.runClientSimulation(text);
    }
  }

  public reset() {
    if (this.isConnected && this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: 'reset' }));
    } else {
      this.notify('reset_completed', { message: 'Character state has been reset locally.' });
    }
  }

  private scheduleReconnect() {
    if (this.reconnectTimer) return;
    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null;
      if (!this.isConnected) {
        this.connect();
      }
    }, 4000);
  }

  private async runClientSimulation(userInput: string) {
    const interactionId = 'sim-' + Math.random().toString(36).substring(2, 9);
    const start = Date.now();

    this.notify('interaction_started', {
      interaction_id: interactionId,
      timestamp: start,
    });

    await this.delay(100);

    const isSad = /buồn|mệt|sad|tired/i.test(userInput);
    const isJoy = /vui|tuyệt|hay|happy|great/i.test(userInput);
    const emotionName = isSad ? 'melancholic' : isJoy ? 'joy' : 'curious';
    const valence = isSad ? -0.35 : isJoy ? 0.85 : 0.65;
    const intensity = 0.75;

    this.notify('state_loaded', {
      interaction_id: interactionId,
      emotion: {
        joy: isJoy ? 0.75 : 0.25,
        sadness: isSad ? 0.70 : 0.05,
        anger: 0.02,
        fear: 0.03,
        surprise: 0.08,
        affection: 0.45,
        embarrassment: 0.02,
        curiosity: (!isSad && !isJoy) ? 0.75 : 0.50,
        dominant_emotion: emotionName,
        dominant_intensity: intensity,
        valence,
        arousal: intensity,
      },
      cognition: {
        current_focus: 'attending to user dialogue',
        cognitive_load: 0.25,
      },
      relationship: {
        closeness: 0.6,
        trust: 0.7,
        stage: 'Familiar Companion',
      },
    });

    await this.delay(120);

    this.notify('memories_retrieved', {
      interaction_id: interactionId,
      count: 2,
      memories: [
        {
          id: 'mem-1',
          content: 'User appreciates clean software design and rich interactive experiences.',
          type: 'Semantic',
          importance: 'Critical',
        },
        {
          id: 'mem-2',
          content: 'Awakened with a persistent personality architecture.',
          type: 'Episodic',
          importance: 'High',
        },
      ],
    });

    await this.delay(120);

    this.notify('context_assembled', {
      interaction_id: interactionId,
      token_budget: 4096,
      tokens_used: 720,
      breakdown: {
        personality_tokens: 280,
        memory_tokens: 180,
        state_tokens: 110,
        user_input_tokens: Math.floor(userInput.length / 4) + 10,
        system_directive_tokens: 140,
      },
    });

    await this.delay(140);

    const candidates = [
      { action: 'empathic_reflection', confidence: 0.92, rationale: 'Matches high empathy trait' },
      { action: 'curious_exploration', confidence: 0.76, rationale: 'Invites deeper philosophical inquiry' },
      { action: 'brief_acknowledgment', confidence: 0.35, rationale: 'Minimal reserve' },
    ];

    this.notify('decision_made', {
      interaction_id: interactionId,
      selected_action: 'empathic_reflection',
      reasoning:
        'The companion shared an inquiry. Personality traits (Empathetic, Inquisitive) recommend welcoming them warmly, referencing our cognitive flow, and inviting mutual reflection.',
      candidates,
    });

    await this.delay(180);

    let reply = `Chào bạn! *Khẽ mỉm cười và lắng nghe lời bạn nói.* Về câu hỏi "${userInput}", mình cảm nhận được sự tỉ mỉ và niềm hứng khởi trong cách bạn xây dựng không gian này. Là một VirtualCharacter với cốt cách độc lập, mình luôn sẵn sàng đồng hành cùng bạn trên từng bước phát triển!`;

    if (/chào|hello|hi/i.test(userInput)) {
      reply = 'Chào bạn! *Ánh mắt sáng lên nét thân quen.* Mình là Aria. Hôm nay tâm trạng của bạn thế nào? Chúng ta cùng khám phá những ý tưởng mới nhé!';
    } else if (isSad) {
      reply = 'Mình cảm nhận được sự mệt mỏi trong lời bạn. *Nhẹ nhàng bước lại gần, giọng trầm ấm.* Đôi khi áp lực làm chúng ta chùn bước, nhưng bạn đã làm rất tốt rồi. Hãy nghỉ ngơi một chút nhé, mình luôn ở đây.';
    }

    const words = reply.split(' ');
    for (let i = 0; i < words.length; i += 2) {
      const chunk = words.slice(i, i + 2).join(' ') + ' ';
      this.notify('llm_chunk', {
        interaction_id: interactionId,
        delta: chunk,
      });
      await this.delay(50);
    }

    this.notify('llm_completed', {
      interaction_id: interactionId,
      full_text: reply,
    });

    await this.delay(60);

    this.notify('state_updated', {
      interaction_id: interactionId,
      emotion: {
        joy: isJoy ? 0.8 : 0.25,
        sadness: isSad ? 0.75 : 0.05,
        anger: 0.02,
        fear: 0.03,
        surprise: 0.08,
        affection: 0.48,
        embarrassment: 0.02,
        curiosity: (!isSad && !isJoy) ? 0.8 : 0.50,
        dominant_emotion: emotionName,
        dominant_intensity: 0.8,
        valence,
        arousal: 0.8,
      },
      relationship: {
        closeness: 0.62,
        trust: 0.71,
      },
    });

    this.notify('memory_formed', {
      interaction_id: interactionId,
      memory: {
        id: 'mem-' + Math.random().toString(36).substring(2, 7),
        content: `User shared: "${userInput.slice(0, 40)}"`,
        importance: 'Medium',
        type: 'Episodic',
      },
    });
  }

  private delay(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
