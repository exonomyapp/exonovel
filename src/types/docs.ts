export interface DocHeading {
  id: string;
  text: string;
  level: number;
}

export interface DocNode {
  id: string;
  name: string;
  title: string;
  path: string;
  type: 'file' | 'folder';
  updatedAt?: string;
  wordCount?: number;
  headings?: DocHeading[];
  children?: DocNode[];
}

export interface DocPayload {
  path: string;
  name: string;
  title: string;
  content: string;
  updatedAt: string;
  wordCount: number;
  readingMinutes: number;
  headings: DocHeading[];
}
