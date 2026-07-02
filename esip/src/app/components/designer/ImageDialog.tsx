import { useState, useRef } from 'react';
import type { Editor } from '@tiptap/react';
import { ImageIcon, Upload, Link, X, Loader2 } from 'lucide-react';
import { Button } from '../ui/button';
import { cn } from '../ui/utils';

interface ImageDialogProps {
  editor: Editor;
  open: boolean;
  onClose: () => void;
}

export function ImageDialog({ editor, open, onClose }: ImageDialogProps) {
  const [tab, setTab] = useState<'url' | 'upload'>('url');
  const [url, setUrl] = useState('');
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState('');
  const fileRef = useRef<HTMLInputElement>(null);

  if (!open) return null;

  const insertImage = (src: string, alt?: string) => {
    editor.chain().focus().setImage({ src, alt: alt ?? '' }).run();
    onClose();
    setUrl('');
    setError('');
  };

  const handleUrlInsert = () => {
    const trimmed = url.trim();
    if (!trimmed) { setError('Please enter an image URL'); return; }
    insertImage(trimmed);
  };

  const handleFileUpload = (file: File) => {
    if (!file.type.startsWith('image/')) { setError('Please select an image file (PNG, JPG, GIF, WebP)'); return; }
    if (file.size > 5 * 1024 * 1024) { setError('File must be under 5 MB'); return; }
    setUploading(true);
    setError('');
    const reader = new FileReader();
    reader.onload = (e) => {
      insertImage(e.target?.result as string, file.name);
      setUploading(false);
    };
    reader.onerror = () => { setError('Failed to read file'); setUploading(false); };
    reader.readAsDataURL(file);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm"
      onMouseDown={(e) => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div className="w-[440px] rounded-xl border border-border bg-card shadow-2xl overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-border">
          <div className="flex items-center gap-2">
            <div className="flex size-7 items-center justify-center rounded-lg bg-primary/10">
              <ImageIcon className="size-4 text-primary" />
            </div>
            <span className="font-semibold text-sm">Insert Image</span>
          </div>
          <button
            onMouseDown={onClose}
            className="rounded p-1 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
          >
            <X className="size-4" />
          </button>
        </div>

        {/* Tabs */}
        <div className="flex border-b border-border">
          {(['url', 'upload'] as const).map(t => (
            <button
              key={t}
              onMouseDown={() => { setTab(t); setError(''); }}
              className={cn(
                'flex-1 flex items-center justify-center gap-1.5 py-2.5 text-xs font-medium transition-colors',
                tab === t
                  ? 'text-primary border-b-2 border-primary bg-primary/5'
                  : 'text-muted-foreground hover:text-foreground',
              )}
            >
              {t === 'url' ? <><Link className="size-3" />From URL</> : <><Upload className="size-3" />Upload file</>}
            </button>
          ))}
        </div>

        {/* Body */}
        <div className="p-4">
          {tab === 'url' ? (
            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-muted-foreground mb-1 block">Image URL</label>
                <input
                  autoFocus
                  type="url"
                  value={url}
                  onChange={(e) => { setUrl(e.target.value); setError(''); }}
                  onKeyDown={(e) => { if (e.key === 'Enter') handleUrlInsert(); if (e.key === 'Escape') onClose(); }}
                  placeholder="https://example.com/image.png"
                  className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary/40"
                />
              </div>
              {error && <p className="text-xs text-destructive">{error}</p>}
              <div className="flex justify-end gap-2">
                <Button variant="outline" size="sm" onMouseDown={onClose} className="h-8 text-xs">Cancel</Button>
                <Button size="sm" onMouseDown={handleUrlInsert} className="h-8 text-xs">Insert</Button>
              </div>
            </div>
          ) : (
            <div className="space-y-3">
              <div
                onMouseDown={() => !uploading && fileRef.current?.click()}
                onDragOver={(e) => e.preventDefault()}
                onDrop={(e) => { e.preventDefault(); const f = e.dataTransfer.files[0]; if (f) handleFileUpload(f); }}
                className={cn(
                  'flex flex-col items-center justify-center gap-2 rounded-xl border-2 border-dashed py-10 transition-colors',
                  uploading
                    ? 'border-border bg-muted/30 cursor-wait'
                    : 'border-border bg-muted/30 cursor-pointer hover:border-primary/50 hover:bg-primary/5',
                )}
              >
                {uploading ? (
                  <Loader2 className="size-8 text-muted-foreground animate-spin" />
                ) : (
                  <>
                    <Upload className="size-8 text-muted-foreground" />
                    <p className="text-sm text-muted-foreground">Click or drag & drop an image</p>
                    <p className="text-xs text-muted-foreground/60">PNG, JPG, GIF, WebP · max 5 MB</p>
                  </>
                )}
              </div>
              <input
                ref={fileRef}
                type="file"
                accept="image/*"
                className="hidden"
                onChange={(e) => { const f = e.target.files?.[0]; if (f) handleFileUpload(f); e.target.value = ''; }}
              />
              {error && <p className="text-xs text-destructive">{error}</p>}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
