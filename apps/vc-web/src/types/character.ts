export interface EmotionData {
  primary_emotion: string;
  intensity: number;
  valence: number;
  arousal: number;
}

export interface RelationshipData {
  closeness: number;
  trust: number;
  stage: string;
  known_facts: string[];
}

export interface PersonalityData {
  identity: {
    core_identity: string;
    background: string;
  };
  traits: string[];
  values: string[];
  preferences: string[];
  behavior_tendencies: string[];
  communication_style: {
    tone: string;
    quirks: string[];
  };
  decision_tendencies: {
    risk_tolerance: string;
    primary_drivers: string[];
  };
  boundaries: string[];
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
