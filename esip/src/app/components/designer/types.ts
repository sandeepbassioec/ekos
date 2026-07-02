import type { JSONContent } from '@tiptap/react';
import type { DesignerDocumentCategory } from '../../lib/db/types';

export interface DesignerTemplate {
  id: string;
  name: string;
  description: string;
  category: DesignerDocumentCategory;
  badge: string;
  initialContent: JSONContent;
  systemPrompt: string;
  toolboxActions: string[];
}

export interface WidgetTarget {
  pos: number;
  nodeType: string;
  attrs: Record<string, string>;
  targetField: string;
  displayName: string;
}

export interface ToolboxAction {
  id: string;
  label: string;
  description: string;
  placeholder: string;
  buildPrompt: (userInput: string, surroundingText: string, template: DesignerTemplate) => string;
}
