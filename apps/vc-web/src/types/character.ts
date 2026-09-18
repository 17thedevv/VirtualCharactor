export interface EmotionData {
  // Multi-axis emotion scores [0, 1]
  joy: number;
  sadness: number;
  anger: number;
  fear: number;
  surprise: number;
  affection: number;
  embarrassment: number;
  curiosity: number;
  // Computed values
  dominant_emotion: string;
  dominant_intensity: number;
  valence: number;
  arousal: number;
}

export interface RelationshipData {
  stage: string;
  closeness: number;
  trust: number;
  familiarity?: number;
  affection?: number;
  tension?: number;
  known_facts: string[];
}

export interface TraitScores {
  playfulness: number;
  empathy: number;
  curiosity: number;
  assertiveness: number;
  patience: number;
  custom?: Record<string, number>;
}

export interface ValueItemData {
  name: string;
  importance: number;
  description?: string;
}

export interface PersonalityGoalData {
  id: string;
  description: string;
  priority: 'low' | 'medium' | 'high';
}

export interface PersonalityData {
  id?: string;
  identity: {
    name?: string;
    role?: string;
    core_identity: string;
    background: string;
    age_representation?: string;
  };
  traits: TraitScores;
  values: {
    items: ValueItemData[];
  };
  preferences: {
    likes: string[];
    dislikes: string[];
  };
  behavior_tendencies: {
    humor: 'low' | 'medium' | 'high';
    teasing: 'low' | 'medium' | 'high';
    initiative: 'low' | 'medium' | 'high';
    emotional_expression: 'low' | 'medium' | 'high';
    conflict_avoidance: 'low' | 'medium' | 'high';
  };
  communication_style: {
    formality: 'low' | 'medium' | 'high';
    verbosity: 'low' | 'medium' | 'high';
    emotionality: 'low' | 'medium' | 'high';
    emoji_usage: 'low' | 'medium' | 'high';
    humor: 'low' | 'medium' | 'high';
    directness: 'low' | 'medium' | 'high';
    tone: string;
    quirks: string[];
  };
  decision_tendencies: {
    prioritize_user_comfort: 'low' | 'medium' | 'high';
    prioritize_truth: 'low' | 'medium' | 'high';
    avoid_unnecessary_conflict: 'low' | 'medium' | 'high';
    take_initiative: 'low' | 'medium' | 'high';
    risk_tolerance: 'low' | 'medium' | 'high';
    primary_drivers: string[];
  };
  boundaries: {
    avoid: string[];
    preserve: string[];
  };
  goals: {
    items: PersonalityGoalData[];
  };
}

export interface MemoryItem {
  id: string;
  content: string;
  type: 'Episodic' | 'Semantic' | 'Procedural' | string;
  importance: 'Low' | 'Medium' | 'High' | 'Critical' | string;
}

export interface DecisionCandidate {
  action: string;
  confidence: number;
  rationale?: string;
}

export interface DecisionTraceData {
  selected_action: string;
  reasoning: string;
  candidates: DecisionCandidate[];
}

export interface ContextBreakdownData {
  token_budget: number;
  tokens_used: number;
  breakdown: {
    personality_tokens: number;
    memory_tokens: number;
    state_tokens: number;
    user_input_tokens: number;
    system_directive_tokens: number;
  };
}

export interface ChatMessage {
  id: string;
  sender: 'user' | 'character';
  text: string;
  timestamp: number;
  isStreaming?: boolean;
  decisionTrace?: DecisionTraceData;
  contextBreakdown?: ContextBreakdownData;
}
