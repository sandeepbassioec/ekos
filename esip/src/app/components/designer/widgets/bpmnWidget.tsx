import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useEffect, useRef, useContext, useCallback } from 'react';
import { GripVertical, Pencil, Check, X, Download, Sparkles } from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

// ── Default BPMN XML ──────────────────────────────────────────────────────────

const DEFAULT_BPMN = `<?xml version="1.0" encoding="UTF-8"?>
<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
             xmlns:bpmndi="http://www.omg.org/spec/BPMN/20100524/DI"
             xmlns:dc="http://www.omg.org/spec/DD/20100524/DC"
             xmlns:di="http://www.omg.org/spec/DD/20100524/DI"
             id="Definitions_1" targetNamespace="http://bpmn.io/schema/bpmn">
  <process id="Process_1" isExecutable="false">
    <startEvent id="StartEvent_1" name="Start">
      <outgoing>Flow_1</outgoing>
    </startEvent>
    <task id="Task_1" name="Process Request">
      <incoming>Flow_1</incoming>
      <outgoing>Flow_2</outgoing>
    </task>
    <exclusiveGateway id="Gateway_1" name="Valid?">
      <incoming>Flow_2</incoming>
      <outgoing>Flow_3</outgoing>
      <outgoing>Flow_4</outgoing>
    </exclusiveGateway>
    <task id="Task_2" name="Handle Error">
      <incoming>Flow_3</incoming>
      <outgoing>Flow_5</outgoing>
    </task>
    <endEvent id="EndEvent_1" name="End">
      <incoming>Flow_4</incoming>
      <incoming>Flow_5</incoming>
    </endEvent>
    <sequenceFlow id="Flow_1" sourceRef="StartEvent_1" targetRef="Task_1"/>
    <sequenceFlow id="Flow_2" sourceRef="Task_1" targetRef="Gateway_1"/>
    <sequenceFlow id="Flow_3" name="No" sourceRef="Gateway_1" targetRef="Task_2"/>
    <sequenceFlow id="Flow_4" name="Yes" sourceRef="Gateway_1" targetRef="EndEvent_1"/>
    <sequenceFlow id="Flow_5" sourceRef="Task_2" targetRef="EndEvent_1"/>
  </process>
  <bpmndi:BPMNDiagram id="BPMNDiagram_1">
    <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="Process_1">
      <bpmndi:BPMNShape id="StartEvent_1_di" bpmnElement="StartEvent_1">
        <dc:Bounds x="152" y="102" width="36" height="36"/>
        <bpmndi:BPMNLabel><dc:Bounds x="155" y="145" width="30" height="14"/></bpmndi:BPMNLabel>
      </bpmndi:BPMNShape>
      <bpmndi:BPMNShape id="Task_1_di" bpmnElement="Task_1">
        <dc:Bounds x="240" y="80" width="100" height="80"/>
      </bpmndi:BPMNShape>
      <bpmndi:BPMNShape id="Gateway_1_di" bpmnElement="Gateway_1" isMarkerVisible="true">
        <dc:Bounds x="395" y="95" width="50" height="50"/>
        <bpmndi:BPMNLabel><dc:Bounds x="398" y="71" width="44" height="14"/></bpmndi:BPMNLabel>
      </bpmndi:BPMNShape>
      <bpmndi:BPMNShape id="Task_2_di" bpmnElement="Task_2">
        <dc:Bounds x="500" y="160" width="100" height="80"/>
      </bpmndi:BPMNShape>
      <bpmndi:BPMNShape id="EndEvent_1_di" bpmnElement="EndEvent_1">
        <dc:Bounds x="552" y="102" width="36" height="36"/>
        <bpmndi:BPMNLabel><dc:Bounds x="558" y="145" width="25" height="14"/></bpmndi:BPMNLabel>
      </bpmndi:BPMNShape>
      <bpmndi:BPMNEdge id="Flow_1_di" bpmnElement="Flow_1">
        <di:waypoint x="188" y="120"/><di:waypoint x="240" y="120"/>
      </bpmndi:BPMNEdge>
      <bpmndi:BPMNEdge id="Flow_2_di" bpmnElement="Flow_2">
        <di:waypoint x="340" y="120"/><di:waypoint x="395" y="120"/>
      </bpmndi:BPMNEdge>
      <bpmndi:BPMNEdge id="Flow_3_di" bpmnElement="Flow_3">
        <di:waypoint x="420" y="145"/><di:waypoint x="420" y="200"/><di:waypoint x="500" y="200"/>
      </bpmndi:BPMNEdge>
      <bpmndi:BPMNEdge id="Flow_4_di" bpmnElement="Flow_4">
        <di:waypoint x="445" y="120"/><di:waypoint x="552" y="120"/>
      </bpmndi:BPMNEdge>
      <bpmndi:BPMNEdge id="Flow_5_di" bpmnElement="Flow_5">
        <di:waypoint x="600" y="160"/><di:waypoint x="570" y="138"/>
      </bpmndi:BPMNEdge>
    </bpmndi:BPMNPlane>
  </bpmndi:BPMNDiagram>
</definitions>`;

// ── Widget view ───────────────────────────────────────────────────────────────

function BpmnView({ node, updateAttributes, selected, getPos }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const [isEditing, setIsEditing] = useState(false);
  const [error, setError] = useState('');
  const viewerRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<HTMLDivElement>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const modelerInstanceRef = useRef<any>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const viewerInstanceRef = useRef<any>(null);

  const xml = node.attrs.xml || DEFAULT_BPMN;

  // Render read-only viewer
  useEffect(() => {
    if (!viewerRef.current || isEditing) return;
    let cancelled = false;

    (async () => {
      try {
        const { default: BpmnViewer } = await import('bpmn-js/lib/NavigatedViewer');
        if (cancelled) return;

        if (viewerInstanceRef.current) {
          viewerInstanceRef.current.destroy();
          viewerInstanceRef.current = null;
        }

        const viewer = new BpmnViewer({ container: viewerRef.current });
        viewerInstanceRef.current = viewer;
        await viewer.importXML(xml);
        viewer.get('canvas').zoom('fit-viewport', 'auto');
        setError('');
      } catch (e) {
        if (!cancelled) setError((e as Error).message || 'Invalid BPMN');
      }
    })();

    return () => { cancelled = true; };
  }, [xml, isEditing]);

  // Mount modeler in edit mode
  useEffect(() => {
    if (!editorRef.current || !isEditing) return;
    let cancelled = false;

    (async () => {
      try {
        const { default: BpmnModeler } = await import('bpmn-js/lib/Modeler');
        if (cancelled) return;

        if (modelerInstanceRef.current) {
          modelerInstanceRef.current.destroy();
          modelerInstanceRef.current = null;
        }

        const modeler = new BpmnModeler({ container: editorRef.current });
        modelerInstanceRef.current = modeler;
        await modeler.importXML(xml);
        modeler.get('canvas').zoom('fit-viewport', 'auto');
        setError('');
      } catch (e) {
        if (!cancelled) setError((e as Error).message || 'Failed to load modeler');
      }
    })();

    return () => {
      cancelled = true;
      if (modelerInstanceRef.current) {
        modelerInstanceRef.current.destroy();
        modelerInstanceRef.current = null;
      }
    };
  }, [isEditing]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleSave = useCallback(async () => {
    if (!modelerInstanceRef.current) return;
    try {
      const { xml: savedXml } = await modelerInstanceRef.current.saveXML({ format: true });
      updateAttributes({ xml: savedXml });
      setIsEditing(false);
    } catch (e) {
      setError((e as Error).message || 'Failed to save');
    }
  }, [updateAttributes]);

  const handleCancel = useCallback(() => {
    setIsEditing(false);
    setError('');
  }, []);

  const handleExportSvg = useCallback(async () => {
    const instance = isEditing ? modelerInstanceRef.current : viewerInstanceRef.current;
    if (!instance) return;
    try {
      const { svg } = await instance.saveSVG();
      const blob = new Blob([svg], { type: 'image/svg+xml' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${node.attrs.title || 'diagram'}.svg`;
      a.click();
      URL.revokeObjectURL(url);
    } catch { /* ignore */ }
  }, [isEditing, node.attrs.title]);

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
          <span className="text-[9px] font-bold text-orange-700 dark:text-orange-400 bg-orange-100 dark:bg-orange-900/40 px-2 py-0.5 rounded">BPMN</span>
          {isPreview ? (
            <span className="flex-1 text-sm font-semibold">{node.attrs.title || 'Process Diagram'}</span>
          ) : (
            <input
              value={node.attrs.title || ''}
              onChange={(e) => updateAttributes({ title: e.target.value })}
              placeholder="Process diagram title…"
              className="flex-1 text-sm font-semibold bg-transparent focus:outline-none placeholder:text-muted-foreground"
            />
          )}
          <div className="ml-auto flex items-center gap-1">
            {!isPreview && (
              <button
                onMouseDown={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  document.dispatchEvent(new CustomEvent('ask-sandeep-target', {
                    bubbles: true,
                    detail: { pos: getPos?.(), nodeType: 'bpmn', attrs: { ...node.attrs }, targetField: 'xml', displayName: `BPMN: ${node.attrs.title || 'Untitled'}` },
                  }));
                }}
                className="opacity-0 group-hover:opacity-70 hover:opacity-100 flex items-center gap-1 text-[10px] text-primary border border-primary/30 bg-primary/5 rounded px-2 py-0.5 transition-all"
                title="Generate BPMN with AI"
              >
                <Sparkles className="size-3" /> AI
              </button>
            )}
            <button onClick={handleExportSvg} className="opacity-0 group-hover:opacity-60 hover:opacity-100 p-1 text-muted-foreground hover:text-foreground transition-all" title="Export SVG">
              <Download className="size-3.5" />
            </button>
            {!isPreview && !isEditing && (
              <button onClick={() => setIsEditing(true)} className="flex items-center gap-1 text-[10px] border border-border rounded px-2 py-0.5 text-muted-foreground hover:text-foreground transition-colors">
                <Pencil className="size-3" /> Edit
              </button>
            )}
            {!isPreview && isEditing && (
              <>
                <button onClick={handleSave} className="flex items-center gap-1 text-[10px] bg-primary text-primary-foreground rounded px-2.5 py-1 font-medium">
                  <Check className="size-3" /> Save
                </button>
                <button onClick={handleCancel} className="flex items-center gap-1 text-[10px] border border-border rounded px-2.5 py-1 text-muted-foreground hover:text-foreground">
                  <X className="size-3" /> Cancel
                </button>
              </>
            )}
          </div>
        </div>

        {error && (
          <div className="px-4 py-2 text-xs text-destructive bg-destructive/10 border-b border-destructive/20">{error}</div>
        )}

        {/* Viewer (read-only) */}
        {!isEditing && (
          <div ref={viewerRef} style={{ height: 320 }} className="bpmn-container" />
        )}

        {/* Modeler (edit mode) */}
        {isEditing && (
          <div ref={editorRef} style={{ height: 480 }} className="bpmn-container" />
        )}
      </div>
    </NodeViewWrapper>
  );
}

// ── Tiptap extension ──────────────────────────────────────────────────────────

export const BpmnExtension = Node.create({
  name: 'bpmn',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      title: { default: '' },
      xml:   { default: DEFAULT_BPMN },
    };
  },

  parseHTML()  { return [{ tag: 'div[data-type="bpmn"]' }]; },
  renderHTML({ HTMLAttributes }) { return ['div', mergeAttributes({ 'data-type': 'bpmn' }, HTMLAttributes)]; },
  addNodeView() { return ReactNodeViewRenderer(BpmnView); },
});
