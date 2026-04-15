export interface Lesson {
  id: number;
  character: string;
  code: string;
  description: string;
}

export interface NewLesson {
  character: string;
  code: string;
  description: string;
}

export interface ProgressUpdate {
  user_name: string;
  lesson_id: number;
  accuracy: number;
  score: number;
}
