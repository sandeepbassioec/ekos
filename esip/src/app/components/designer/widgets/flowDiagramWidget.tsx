/**
 * Flow Diagram widget — powered by @maxgraph/core (draw.io engine).
 * Replaces the xyflow-based widget with a richer shape library, orthogonal
 * routing, and draw.io-compatible XML persistence.
 */
import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useEffect, useRef, useContext } from 'react';
import {
  Graph, InternalEvent, ModelXmlSerializer,
  type Cell,
} from '@maxgraph/core';
import type { CellStyle } from '@maxgraph/core';
import {
  GripVertical, Pencil, Check, X,
  ZoomIn, ZoomOut, Maximize2, Trash2,
} from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

// ── Shape palette ─────────────────────────────────────────────────────────────
interface ShapeDef {
  key: string;
  label: string;
  emoji: string;
  w: number;
  h: number;
  style: CellStyle;
}

const SHAPES: ShapeDef[] = [
  {
    key: 'terminal', label: 'Start / End', emoji: '⬭', w: 120, h: 40,
    style: { shape: 'ellipse', whiteSpace: 'wrap', fillColor: '#d5e8d4', strokeColor: '#82b366', fontStyle: 1 },
  },
  {
    key: 'process', label: 'Process', emoji: '▭', w: 160, h: 45,
    style: { rounded: true, whiteSpace: 'wrap', fillColor: '#dae8fc', strokeColor: '#6c8ebf' },
  },
  {
    key: 'decision', label: 'Decision', emoji: '◇', w: 140, h: 70,
    style: { shape: 'rhombus', whiteSpace: 'wrap', fillColor: '#fff2cc', strokeColor: '#d6b656' },
  },
  {
    key: 'data', label: 'Data / I/O', emoji: '▱', w: 150, h: 45,
    style: { shape: 'parallelogram' as any, whiteSpace: 'wrap', fillColor: '#e1d5e7', strokeColor: '#9673a6' },
  },
  {
    key: 'database', label: 'Database', emoji: '⬡', w: 100, h: 60,
    style: { shape: 'cylinder', whiteSpace: 'wrap', fillColor: '#e1d5e7', strokeColor: '#9673a6' },
  },
  {
    key: 'cloud', label: 'Cloud', emoji: '☁', w: 130, h: 70,
    style: { shape: 'cloud', whiteSpace: 'wrap', fillColor: '#dae8fc', strokeColor: '#6c8ebf' },
  },
  {
    key: 'hexagon', label: 'Hexagon', emoji: '⬡', w: 120, h: 70,
    style: { shape: 'hexagon', whiteSpace: 'wrap', fillColor: '#fff2cc', strokeColor: '#d6b656' },
  },
  {
    key: 'actor', label: 'Actor', emoji: '👤', w: 50, h: 70,
    style: { shape: 'actor', whiteSpace: 'wrap', fillColor: '#dae8fc', strokeColor: '#6c8ebf' },
  },
  {
    key: 'triangle', label: 'Triangle', emoji: '▲', w: 80, h: 80,
    style: { shape: 'triangle', whiteSpace: 'wrap', fillColor: '#f8cecc', strokeColor: '#b85450' },
  },
  {
    key: 'note', label: 'Note', emoji: '📝', w: 130, h: 50,
    style: { shape: 'note' as any, whiteSpace: 'wrap', fillColor: '#fffde7', strokeColor: '#ccc' },
  },
];

// ── Colour swatches for quick node recolouring ────────────────────────────────
const FILL_COLORS: string[] = [
  '#ffffff', '#dae8fc', '#d5e8d4', '#fff2cc',
  '#f8cecc', '#e1d5e7', '#fffde7', '#e6f3ff',
];

// ── Default diagram XML (built programmatically on first use) ─────────────────
function buildDefaultDiagram(graph: Graph): void {
  const parent = graph.getDefaultParent();
  graph.batchUpdate(() => {
    const start = graph.insertVertex(
      parent, 'n1', 'Start', 190, 30, 120, 40,
      { shape: 'ellipse', whiteSpace: 'wrap', fillColor: '#d5e8d4', strokeColor: '#82b366', fontStyle: 1 },
    );
    const proc = graph.insertVertex(
      parent, 'n2', 'Process Request', 155, 120, 185, 45,
      { rounded: true, whiteSpace: 'wrap', fillColor: '#dae8fc', strokeColor: '#6c8ebf' },
    );
    const dec = graph.insertVertex(
      parent, 'n3', 'Valid?', 170, 220, 155, 75,
      { shape: 'rhombus', whiteSpace: 'wrap', fillColor: '#fff2cc', strokeColor: '#d6b656' },
    );
    const err = graph.insertVertex(
      parent, 'n4', 'Error Response', 20, 360, 145, 45,
      { rounded: true, whiteSpace: 'wrap', fillColor: '#f8cecc', strokeColor: '#b85450' },
    );
    const end = graph.insertVertex(
      parent, 'n5', 'End', 315, 360, 120, 40,
      { shape: 'ellipse', whiteSpace: 'wrap', fillColor: '#d5e8d4', strokeColor: '#82b366', fontStyle: 1 },
    );
    graph.insertEdge(parent, 'e1', '', start, proc, { edgeStyle: 'orthogonalEdgeStyle' } as CellStyle);
    graph.insertEdge(parent, 'e2', '', proc, dec, { edgeStyle: 'orthogonalEdgeStyle' } as CellStyle);
    graph.insertEdge(parent, 'e3', 'No', dec, err,
      { edgeStyle: 'orthogonalEdgeStyle', exitX: 0, exitY: 0.5, entryX: 1, entryY: 0.5 } as CellStyle);
    graph.insertEdge(parent, 'e4', 'Yes', dec, end,
      { edgeStyle: 'orthogonalEdgeStyle', exitX: 1, exitY: 0.5, entryX: 0, entryY: 0.5 } as CellStyle);
  });
}

// ── XML helpers ───────────────────────────────────────────────────────────────
function exportXml(graph: Graph): string {
  return new ModelXmlSerializer(graph.getDataModel()).export({ pretty: false });
}

function loadXml(graph: Graph, xml: string): void {
  try {
    new ModelXmlSerializer(graph.getDataModel()).import(xml);
  } catch {
    buildDefaultDiagram(graph);
  }
}

function fitGraph(graph: Graph): void {
  (graph.getPlugin('fit') as any)?.fit({ margin: 20 });
}

// ── Widget component ──────────────────────────────────────────────────────────
function FlowDiagramView({ node, updateAttributes, selected }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const containerRef = useRef<HTMLDivElement>(null);
  const graphRef = useRef<Graph | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [selectedCell, setSelectedCell] = useState<Cell | null>(null);
  const [labelDraft, setLabelDraft] = useState('');
  const [fillDraft, setFillDraft] = useState('');

  // ── Init graph (mount only) ────────────────────────────────────────────────
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    InternalEvent.disableContextMenu(container);

    const graph = new Graph(container);

    // Base config — always view-only until Edit clicked
    graph.setEnabled(false);
    graph.setConnectable(false);
    graph.setCellsEditable(false);
    graph.setPanning(true);
    (graph as any).panningHandler?.setPinchEnabled?.(false);

    // Style defaults
    const defEdge = graph.getStylesheet().getDefaultEdgeStyle();
    defEdge.edgeStyle = 'orthogonalEdgeStyle' as any;
    defEdge.rounded = true;

    const defVertex = graph.getStylesheet().getDefaultVertexStyle();
    defVertex.fillColor = '#ffffff';
    defVertex.strokeColor = '#aaa';
    defVertex.fontColor = '#333';

    // Load XML or build default
    const xml = node.attrs.xml as string;
    if (xml) {
      loadXml(graph, xml);
    } else {
      buildDefaultDiagram(graph);
    }

    // Fit after a tick so the container has its final size
    setTimeout(() => fitGraph(graph), 50);

    // Selection listener
    graph.getSelectionModel().addListener(InternalEvent.CHANGE, () => {
      const cells = graph.getSelectionCells();
      const v = cells.find(c => c.isVertex()) ?? null;
      setSelectedCell(v);
      if (v) {
        setLabelDraft((graph.getLabel(v) as string) ?? '');
        const style = v.getStyle();
        setFillDraft((style as CellStyle)?.fillColor ?? '');
      } else {
        setLabelDraft('');
        setFillDraft('');
      }
    });

    graphRef.current = graph;
    return () => {
      graph.destroy();
      graphRef.current = null;
    };
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // ── Toggle edit mode ───────────────────────────────────────────────────────
  useEffect(() => {
    const graph = graphRef.current;
    if (!graph) return;
    graph.setEnabled(isEditing);
    graph.setConnectable(isEditing);
    if (!isEditing) {
      graph.clearSelection();
      setSelectedCell(null);
    }
  }, [isEditing]);

  // ── Shape insert ───────────────────────────────────────────────────────────
  const addShape = (shape: ShapeDef) => {
    const graph = graphRef.current;
    if (!graph) return;
    const container = containerRef.current!;
    const view = graph.getView();
    const scale = view.getScale();
    const tr = view.getTranslate();
    const cx = (container.clientWidth / 2 - tr.x) / scale;
    const cy = (container.clientHeight / 2 - tr.y) / scale;
    graph.batchUpdate(() => {
      graph.insertVertex(
        graph.getDefaultParent(), null, shape.label,
        cx - shape.w / 2, cy - shape.h / 2, shape.w, shape.h,
        shape.style,
      );
    });
  };

  // ── Label apply ────────────────────────────────────────────────────────────
  const applyLabel = () => {
    const graph = graphRef.current;
    if (!graph || !selectedCell) return;
    graph.batchUpdate(() => {
      graph.getDataModel().setValue(selectedCell, labelDraft);
    });
  };

  // ── Fill colour ────────────────────────────────────────────────────────────
  const applyFill = (color: string) => {
    const graph = graphRef.current;
    if (!graph || !selectedCell) return;
    setFillDraft(color);
    graph.batchUpdate(() => {
      const currentStyle = selectedCell.getStyle() ?? {};
      graph.getDataModel().setStyle(selectedCell, { ...currentStyle, fillColor: color });
    });
  };

  // ── Delete selected ────────────────────────────────────────────────────────
  const deleteSelected = () => {
    const graph = graphRef.current;
    if (!graph) return;
    const cells = graph.getSelectionCells();
    if (cells.length > 0) {
      graph.removeCells(cells);
      setSelectedCell(null);
    }
  };

  // ── Save / Cancel ──────────────────────────────────────────────────────────
  const handleSave = () => {
    const graph = graphRef.current;
    if (!graph) return;
    updateAttributes({ xml: exportXml(graph) });
    setIsEditing(false);
  };

  const handleCancel = () => {
    const graph = graphRef.current;
    if (!graph) return;
    // Reload from saved XML
    graph.batchUpdate(() => {
      const cells = graph.getChildCells(graph.getDefaultParent(), true, true);
      if (cells.length) graph.removeCells(cells, false);
    });
    const xml = node.attrs.xml as string;
    if (xml) loadXml(graph, xml);
    else buildDefaultDiagram(graph);
    setTimeout(() => fitGraph(graph), 30);
    setIsEditing(false);
    setSelectedCell(null);
  };

  // ── Render ─────────────────────────────────────────────────────────────────
  const editHeight = 520;
  const viewHeight = 320;

  return (
    <NodeViewWrapper>
      <div className={`group relative rounded-xl border border-border bg-card my-3 overflow-hidden shadow-sm ${selected ? 'ring-2 ring-primary/40' : ''}`}>

        {/* ── Header ── */}
        <div className="flex items-center gap-2 px-3 py-2 bg-muted/30 border-b border-border shrink-0">
          {!isPreview && (
            <div data-drag-handle className="w-5 flex items-center justify-center cursor-grab opacity-0 group-hover:opacity-60 transition-opacity">
              <GripVertical className="size-3.5 text-muted-foreground" />
            </div>
          )}
          <span className="text-[9px] font-bold text-blue-700 dark:text-blue-400 bg-blue-100 dark:bg-blue-900/40 px-2 py-0.5 rounded shrink-0">
            FLOW
          </span>
          {isPreview ? (
            <span className="flex-1 text-sm font-semibold truncate">{node.attrs.title || 'Flow Diagram'}</span>
          ) : (
            <input
              value={node.attrs.title || ''}
              onChange={(e) => updateAttributes({ title: e.target.value })}
              placeholder="Diagram title…"
              className="flex-1 text-sm font-semibold bg-transparent focus:outline-none placeholder:text-muted-foreground min-w-0"
            />
          )}

          {!isPreview && !isEditing && (
            <button
              onClick={() => setIsEditing(true)}
              className="ml-auto flex items-center gap-1 text-[10px] text-muted-foreground hover:text-foreground border border-border rounded px-2 py-0.5 transition-colors"
            >
              <Pencil className="size-3" /> Edit
            </button>
          )}
          {!isPreview && isEditing && (
            <div className="ml-auto flex gap-1">
              <button
                onClick={handleSave}
                className="flex items-center gap-1 text-[10px] bg-primary text-primary-foreground rounded px-2.5 py-1 font-medium"
              >
                <Check className="size-3" /> Save
              </button>
              <button
                onClick={handleCancel}
                className="flex items-center gap-1 text-[10px] text-muted-foreground hover:text-foreground border border-border rounded px-2.5 py-1"
              >
                <X className="size-3" /> Cancel
              </button>
            </div>
          )}
        </div>

        {/* ── Body: palette + canvas ── */}
        <div className="flex" style={{ height: isEditing ? editHeight : viewHeight }}>

          {/* Shape palette — edit mode only */}
          {isEditing && (
            <div className="w-[148px] shrink-0 border-r border-border bg-muted/10 flex flex-col overflow-y-auto">
              <div className="p-2 flex flex-col gap-0.5">
                <p className="text-[9px] font-bold uppercase tracking-wider text-muted-foreground mb-1 px-1">Shapes</p>
                {SHAPES.map(s => (
                  <button
                    key={s.key}
                    onClick={() => addShape(s)}
                    className="flex items-center gap-2 text-[11px] px-2 py-1.5 rounded hover:bg-muted transition-colors text-left w-full"
                  >
                    <span className="text-base leading-none shrink-0">{s.emoji}</span>
                    <span className="truncate text-foreground">{s.label}</span>
                  </button>
                ))}

                {/* ── Selected cell panel ── */}
                {selectedCell && (
                  <>
                    <hr className="border-border my-2" />
                    <p className="text-[9px] font-bold uppercase tracking-wider text-muted-foreground mb-1 px-1">Selected</p>

                    {/* Label */}
                    <input
                      value={labelDraft}
                      onChange={e => setLabelDraft(e.target.value)}
                      onBlur={applyLabel}
                      onKeyDown={e => { if (e.key === 'Enter') { e.preventDefault(); applyLabel(); } }}
                      placeholder="Label…"
                      className="text-xs border border-border rounded px-2 py-1.5 bg-background focus:outline-none focus:ring-1 focus:ring-primary/50 w-full"
                    />

                    {/* Fill colour */}
                    <p className="text-[9px] text-muted-foreground mt-1.5 mb-1 px-1">Fill colour</p>
                    <div className="flex flex-wrap gap-1 px-1">
                      {FILL_COLORS.map(c => (
                        <button
                          key={c}
                          onClick={() => applyFill(c)}
                          style={{ background: c }}
                          className={`w-5 h-5 rounded border transition-all ${fillDraft === c ? 'border-primary ring-1 ring-primary ring-offset-1' : 'border-border'}`}
                          title={c}
                        />
                      ))}
                    </div>

                    {/* Delete */}
                    <button
                      onClick={deleteSelected}
                      className="mt-2 flex items-center gap-1.5 text-[11px] text-destructive hover:bg-destructive/10 border border-destructive/30 rounded px-2 py-1 transition-colors w-full"
                    >
                      <Trash2 className="size-3" /> Delete
                    </button>
                  </>
                )}

                {/* ── Zoom controls ── */}
                <hr className="border-border my-2" />
                <p className="text-[9px] font-bold uppercase tracking-wider text-muted-foreground mb-1 px-1">View</p>
                <div className="flex gap-1">
                  <button
                    onClick={() => graphRef.current?.zoomIn()}
                    className="flex-1 flex items-center justify-center py-1 border border-border rounded hover:bg-muted transition-colors"
                    title="Zoom in"
                  >
                    <ZoomIn className="size-3.5" />
                  </button>
                  <button
                    onClick={() => graphRef.current?.zoomOut()}
                    className="flex-1 flex items-center justify-center py-1 border border-border rounded hover:bg-muted transition-colors"
                    title="Zoom out"
                  >
                    <ZoomOut className="size-3.5" />
                  </button>
                  <button
                    onClick={() => { const g = graphRef.current; if (g) fitGraph(g); }}
                    className="flex-1 flex items-center justify-center py-1 border border-border rounded hover:bg-muted transition-colors"
                    title="Fit to screen"
                  >
                    <Maximize2 className="size-3" />
                  </button>
                </div>

                <p className="text-[9px] text-muted-foreground leading-tight px-1 mt-2">
                  Drag nodes · Connect handles · Click to select · Del removes
                </p>
              </div>
            </div>
          )}

          {/* maxGraph canvas */}
          <div
            ref={containerRef}
            className="flex-1 bg-background overflow-hidden"
            style={{ minWidth: 0, cursor: isEditing ? 'default' : 'grab' }}
            onKeyDown={e => { if (isEditing && e.key === 'Delete') deleteSelected(); }}
            tabIndex={isEditing ? 0 : -1}
          />
        </div>
      </div>
    </NodeViewWrapper>
  );
}

// ── Tiptap Extension ──────────────────────────────────────────────────────────
export const FlowDiagramExtension = Node.create({
  name: 'flowDiagram',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      title: { default: '' },
      xml:   { default: '' },
      // kept for backwards-compat reading of old docs — ignored on render
      nodes: { default: '' },
      edges: { default: '' },
    };
  },

  parseHTML()  { return [{ tag: 'div[data-type="flow-diagram"]' }]; },
  renderHTML({ HTMLAttributes }) {
    return ['div', mergeAttributes({ 'data-type': 'flow-diagram' }, HTMLAttributes)];
  },
  addNodeView() { return ReactNodeViewRenderer(FlowDiagramView); },
});
