import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useContext, useCallback, useRef, useEffect } from 'react';
import { GripVertical, Plus, Trash2, ChevronDown } from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

// ── Preset definitions ────────────────────────────────────────────────────────

export type MatrixPreset = 'raci' | 'crud' | 'sipoc' | 'bmc' | 'custom';

interface MatrixData {
  headers: string[];
  rows: { label: string; cells: string[] }[];
}

const RACI_VALUES = ['', 'R', 'A', 'C', 'I'];

const PRESETS: Record<MatrixPreset, { label: string; badge: string; badgeColor: string; default: MatrixData }> = {
  raci: {
    label: 'RACI Matrix',
    badge: 'RACI',
    badgeColor: 'text-blue-700 dark:text-blue-400 bg-blue-100 dark:bg-blue-900/40',
    default: {
      headers: ['Activity', 'Product Owner', 'Tech Lead', 'Developer', 'QA'],
      rows: [
        { label: 'Requirements gathering', cells: ['A', 'R', 'C', 'I'] },
        { label: 'Architecture design', cells: ['C', 'R', 'A', 'I'] },
        { label: 'Development', cells: ['I', 'A', 'R', 'C'] },
        { label: 'Testing', cells: ['I', 'C', 'C', 'R'] },
        { label: 'Deployment', cells: ['A', 'R', 'C', 'C'] },
      ],
    },
  },
  crud: {
    label: 'CRUD Matrix',
    badge: 'CRUD',
    badgeColor: 'text-green-700 dark:text-green-400 bg-green-100 dark:bg-green-900/40',
    default: {
      headers: ['Entity', 'Create', 'Read', 'Update', 'Delete'],
      rows: [
        { label: 'User', cells: ['Admin', 'All', 'Self', 'Admin'] },
        { label: 'Project', cells: ['PM', 'Member', 'PM', 'Admin'] },
        { label: 'Document', cells: ['Member', 'Member', 'Author', 'Admin'] },
        { label: 'Report', cells: ['System', 'Manager', '—', '—'] },
      ],
    },
  },
  sipoc: {
    label: 'SIPOC Diagram',
    badge: 'SIPOC',
    badgeColor: 'text-purple-700 dark:text-purple-400 bg-purple-100 dark:bg-purple-900/40',
    default: {
      headers: ['Suppliers', 'Inputs', 'Process', 'Outputs', 'Customers'],
      rows: [
        { label: '', cells: ['', '', '', '', ''] },
        { label: '', cells: ['', '', '', '', ''] },
        { label: '', cells: ['', '', '', '', ''] },
      ],
    },
  },
  bmc: {
    label: 'Business Model Canvas',
    badge: 'BMC',
    badgeColor: 'text-orange-700 dark:text-orange-400 bg-orange-100 dark:bg-orange-900/40',
    default: {
      headers: ['Key Partners', 'Key Activities', 'Value Propositions', 'Customer Relationships', 'Customer Segments', 'Key Resources', '', 'Channels', '', 'Cost Structure', '', 'Revenue Streams', ''],
      rows: [],
    },
  },
  custom: {
    label: 'Custom Grid',
    badge: 'GRID',
    badgeColor: 'text-muted-foreground bg-muted',
    default: {
      headers: ['Column 1', 'Column 2', 'Column 3'],
      rows: [
        { label: 'Row 1', cells: ['', '', ''] },
        { label: 'Row 2', cells: ['', '', ''] },
        { label: 'Row 3', cells: ['', '', ''] },
      ],
    },
  },
};

// ── RACI cell cycler ──────────────────────────────────────────────────────────

const RACI_COLORS: Record<string, string> = {
  R: 'bg-blue-500 text-white font-bold',
  A: 'bg-amber-500 text-white font-bold',
  C: 'bg-green-500 text-white font-bold',
  I: 'bg-slate-400 text-white font-bold',
  '': 'text-muted-foreground',
};

function RaciCell({ value, onChange, readonly }: { value: string; onChange: (v: string) => void; readonly: boolean }) {
  const cycle = () => {
    if (readonly) return;
    const idx = RACI_VALUES.indexOf(value);
    onChange(RACI_VALUES[(idx + 1) % RACI_VALUES.length]);
  };
  return (
    <div
      onClick={cycle}
      className={`w-8 h-8 mx-auto rounded flex items-center justify-center text-xs cursor-pointer select-none transition-colors ${RACI_COLORS[value] || ''} ${!readonly ? 'hover:opacity-80' : ''}`}
    >
      {value || '—'}
    </div>
  );
}

// ── Business Model Canvas layout ──────────────────────────────────────────────

const BMC_BLOCKS = [
  { name: 'Key Partners', row: 1, col: 1, rowSpan: 2, colSpan: 1 },
  { name: 'Key Activities', row: 1, col: 2, rowSpan: 1, colSpan: 1 },
  { name: 'Value Propositions', row: 1, col: 3, rowSpan: 2, colSpan: 1 },
  { name: 'Customer Relationships', row: 1, col: 4, rowSpan: 1, colSpan: 1 },
  { name: 'Customer Segments', row: 1, col: 5, rowSpan: 2, colSpan: 1 },
  { name: 'Key Resources', row: 2, col: 2, rowSpan: 1, colSpan: 1 },
  { name: 'Channels', row: 2, col: 4, rowSpan: 1, colSpan: 1 },
  { name: 'Cost Structure', row: 3, col: 1, rowSpan: 1, colSpan: 2 },
  { name: 'Revenue Streams', row: 3, col: 3, rowSpan: 1, colSpan: 3 },
];

function BmcCanvas({ data, isEditing, onCellChange }: { data: Record<string, string>; isEditing: boolean; onCellChange: (block: string, value: string) => void }) {
  return (
    <div className="grid gap-1 p-2" style={{ display: 'grid', gridTemplateColumns: 'repeat(5, 1fr)', gridTemplateRows: 'repeat(3, minmax(80px, auto))' }}>
      {BMC_BLOCKS.map((block) => (
        <div
          key={block.name}
          className="border border-border rounded p-2 bg-muted/20 min-h-[80px]"
          style={{ gridColumn: `${block.col} / span ${block.colSpan}`, gridRow: `${block.row} / span ${block.rowSpan}` }}
        >
          <div className="text-[9px] font-bold uppercase text-muted-foreground mb-1">{block.name}</div>
          {isEditing ? (
            <textarea
              value={data[block.name] || ''}
              onChange={(e) => onCellChange(block.name, e.target.value)}
              className="w-full text-xs bg-transparent resize-none focus:outline-none min-h-[50px]"
              placeholder="Add notes…"
            />
          ) : (
            <div className="text-xs whitespace-pre-wrap">{data[block.name] || ''}</div>
          )}
        </div>
      ))}
    </div>
  );
}

// ── Main widget view ──────────────────────────────────────────────────────────

function MatrixView({ node, updateAttributes, selected }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const [isEditing, setIsEditing] = useState(false);
  const preset: MatrixPreset = node.attrs.preset || 'custom';
  const presetMeta = PRESETS[preset];

  const parseData = (): MatrixData => {
    try { return JSON.parse(node.attrs.data); } catch { return PRESETS[preset].default; }
  };
  const parseBmc = (): Record<string, string> => {
    try { return JSON.parse(node.attrs.bmcData || '{}'); } catch { return {}; }
  };

  const [localData, setLocalData] = useState<MatrixData>(parseData);
  const [bmcData, setBmcData] = useState<Record<string, string>>(parseBmc);
  const isBmc = preset === 'bmc';
  const isRaci = preset === 'raci';

  // Sync from node attrs when not editing
  const syncRef = useRef(false);
  useEffect(() => {
    if (!isEditing) { setLocalData(parseData()); setBmcData(parseBmc()); }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [node.attrs.data, node.attrs.bmcData, isEditing]);

  const save = useCallback(() => {
    if (isBmc) updateAttributes({ bmcData: JSON.stringify(bmcData) });
    else updateAttributes({ data: JSON.stringify(localData) });
    setIsEditing(false);
    syncRef.current = false;
  }, [isBmc, bmcData, localData, updateAttributes]);

  const cancel = useCallback(() => { setLocalData(parseData()); setBmcData(parseBmc()); setIsEditing(false); }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // Cell update helpers
  const setCell = (ri: number, ci: number, val: string) => setLocalData(d => {
    const rows = d.rows.map((r, i) => i === ri ? { ...r, cells: r.cells.map((c, j) => j === ci ? val : c) } : r);
    return { ...d, rows };
  });
  const setHeader = (ci: number, val: string) => setLocalData(d => ({ ...d, headers: d.headers.map((h, i) => i === ci ? val : h) }));
  const setRowLabel = (ri: number, val: string) => setLocalData(d => ({ ...d, rows: d.rows.map((r, i) => i === ri ? { ...r, label: val } : r) }));
  const addRow = () => setLocalData(d => ({ ...d, rows: [...d.rows, { label: `Row ${d.rows.length + 1}`, cells: new Array(d.headers.length - 1).fill('') }] }));
  const addCol = () => setLocalData(d => ({ headers: [...d.headers, `Col ${d.headers.length}`], rows: d.rows.map(r => ({ ...r, cells: [...r.cells, ''] })) }));
  const delRow = (ri: number) => setLocalData(d => ({ ...d, rows: d.rows.filter((_, i) => i !== ri) }));
  const delCol = (ci: number) => setLocalData(d => ({ headers: d.headers.filter((_, i) => i !== ci + 1), rows: d.rows.map(r => ({ ...r, cells: r.cells.filter((_, i) => i !== ci) })) }));

  const dataHeaders = isBmc ? [] : localData.headers;
  const colHeaders = isBmc ? [] : dataHeaders.slice(1); // first header = row label col

  return (
    <NodeViewWrapper>
      <div className={`group relative rounded-xl border border-border bg-card my-3 overflow-hidden shadow-sm ${selected ? 'ring-2 ring-primary/40' : ''}`}>
        {/* Toolbar */}
        <div className="flex items-center gap-2 px-3 py-2 bg-muted/30 border-b border-border shrink-0">
          {!isPreview && (
            <div className="flex items-center justify-center w-5 cursor-grab opacity-0 group-hover:opacity-60 transition-opacity" data-drag-handle>
              <GripVertical className="size-3.5 text-muted-foreground" />
            </div>
          )}
          <span className={`text-[9px] font-bold px-2 py-0.5 rounded ${presetMeta.badgeColor}`}>{presetMeta.badge}</span>
          {isPreview ? (
            <span className="flex-1 text-sm font-semibold">{node.attrs.title || presetMeta.label}</span>
          ) : (
            <input
              value={node.attrs.title || ''}
              onChange={(e) => updateAttributes({ title: e.target.value })}
              placeholder={presetMeta.label + '…'}
              className="flex-1 text-sm font-semibold bg-transparent focus:outline-none placeholder:text-muted-foreground"
            />
          )}
          {!isPreview && !isEditing && (
            <button onClick={() => setIsEditing(true)} className="ml-auto flex items-center gap-1 text-[10px] border border-border rounded px-2 py-0.5 text-muted-foreground hover:text-foreground transition-colors">
              Edit
            </button>
          )}
          {!isPreview && isEditing && (
            <div className="ml-auto flex items-center gap-1">
              <button onClick={save} className="flex items-center gap-1 text-[10px] bg-primary text-primary-foreground rounded px-2.5 py-1 font-medium">Save</button>
              <button onClick={cancel} className="flex items-center gap-1 text-[10px] border border-border rounded px-2.5 py-1 text-muted-foreground hover:text-foreground">Cancel</button>
            </div>
          )}
        </div>

        {/* BMC Layout */}
        {isBmc && (
          <BmcCanvas data={bmcData} isEditing={isEditing && !isPreview} onCellChange={(block, val) => setBmcData(d => ({ ...d, [block]: val }))} />
        )}

        {/* Table Layout (RACI, CRUD, SIPOC, Custom) */}
        {!isBmc && (
          <div className="overflow-x-auto">
            <table className="w-full text-xs border-collapse">
              <thead>
                <tr className="bg-muted/40">
                  {/* Row label header */}
                  <th className="border border-border px-3 py-2 text-left font-semibold text-muted-foreground min-w-[140px]">
                    {isEditing ? (
                      <input value={dataHeaders[0] || ''} onChange={e => setHeader(0, e.target.value)} className="bg-transparent focus:outline-none w-full font-semibold" />
                    ) : dataHeaders[0]}
                  </th>
                  {colHeaders.map((h, ci) => (
                    <th key={ci} className="border border-border px-3 py-2 font-semibold text-center relative group/col">
                      {isEditing ? (
                        <div className="flex items-center gap-1">
                          <input value={h} onChange={e => setHeader(ci + 1, e.target.value)} className="bg-transparent focus:outline-none flex-1 text-center font-semibold" />
                          {isEditing && <button onClick={() => delCol(ci)} className="opacity-0 group-hover/col:opacity-60 hover:opacity-100 text-destructive"><Trash2 className="size-2.5" /></button>}
                        </div>
                      ) : h}
                    </th>
                  ))}
                  {isEditing && (
                    <th className="border border-border px-2 py-2">
                      <button onClick={addCol} className="flex items-center justify-center w-full text-muted-foreground hover:text-foreground"><Plus className="size-3" /></button>
                    </th>
                  )}
                </tr>
              </thead>
              <tbody>
                {localData.rows.map((row, ri) => (
                  <tr key={ri} className="hover:bg-muted/20 group/row">
                    <td className="border border-border px-3 py-2 font-medium text-foreground/80">
                      {isEditing ? (
                        <div className="flex items-center gap-1">
                          <input value={row.label} onChange={e => setRowLabel(ri, e.target.value)} className="bg-transparent focus:outline-none flex-1" />
                          <button onClick={() => delRow(ri)} className="opacity-0 group-hover/row:opacity-60 hover:opacity-100 text-destructive"><Trash2 className="size-2.5" /></button>
                        </div>
                      ) : row.label}
                    </td>
                    {row.cells.map((cell, ci) => (
                      <td key={ci} className="border border-border px-3 py-2 text-center">
                        {isRaci ? (
                          <RaciCell value={cell} onChange={v => setCell(ri, ci, v)} readonly={!isEditing || !!isPreview} />
                        ) : isEditing ? (
                          <input value={cell} onChange={e => setCell(ri, ci, e.target.value)} className="bg-transparent focus:outline-none text-center w-full" />
                        ) : (
                          <span className="text-foreground/70">{cell || '—'}</span>
                        )}
                      </td>
                    ))}
                    {isEditing && <td className="border border-border" />}
                  </tr>
                ))}
                {isEditing && (
                  <tr>
                    <td colSpan={localData.headers.length + 1} className="border border-border px-3 py-1">
                      <button onClick={addRow} className="flex items-center gap-1 text-muted-foreground hover:text-foreground text-xs"><Plus className="size-3" /> Add row</button>
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        )}

        {/* RACI legend */}
        {isRaci && (
          <div className="flex items-center gap-3 px-3 py-1.5 border-t border-border bg-muted/10 text-[9px] text-muted-foreground">
            <span className="font-bold">Legend:</span>
            {Object.entries({ R: 'Responsible', A: 'Accountable', C: 'Consulted', I: 'Informed' }).map(([k, v]) => (
              <span key={k}><span className={`inline-flex w-4 h-4 rounded items-center justify-center text-white text-[8px] font-bold mr-0.5 ${RACI_COLORS[k]}`}>{k}</span>{v}</span>
            ))}
            {!isPreview && !isEditing && <span className="ml-auto italic">Click a cell to cycle R→A→C→I</span>}
          </div>
        )}
      </div>
    </NodeViewWrapper>
  );
}

// ── Tiptap extension ──────────────────────────────────────────────────────────

export const MatrixExtension = Node.create({
  name: 'matrix',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      preset:  { default: 'raci' },
      title:   { default: '' },
      data:    { default: JSON.stringify(PRESETS.raci.default) },
      bmcData: { default: '{}' },
    };
  },

  parseHTML()  { return [{ tag: 'div[data-type="matrix"]' }]; },
  renderHTML({ HTMLAttributes }) { return ['div', mergeAttributes({ 'data-type': 'matrix' }, HTMLAttributes)]; },
  addNodeView() { return ReactNodeViewRenderer(MatrixView); },
});

export { PRESETS as MATRIX_PRESETS };
