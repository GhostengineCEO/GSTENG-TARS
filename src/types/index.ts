export interface Message {
  role: 'user' | 'assistant';
  content: string;
  timestamp?: Date;
}

export interface CodeContext {
  filePath: string;
  projectRoot?: string;
  language?: string;
}

export interface AIResponse {
  message: string;
  code?: string;
  metadata?: Record<string, unknown>;
}

export interface VoiceState {
  isRecording: boolean;
  isSpeaking: boolean;
}
