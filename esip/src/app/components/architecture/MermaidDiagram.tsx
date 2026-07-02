import React, { useEffect, useRef, useState } from 'react';
import mermaid from 'mermaid';

interface MermaidDiagramProps {
  chart: string;
  id?: string;
}

export function MermaidDiagram({ chart, id = 'mermaid-diagram' }: MermaidDiagramProps) {
  const elementRef = useRef<HTMLDivElement>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Initialize mermaid
    mermaid.initialize({
      startOnLoad: true,
      theme: 'default',
      securityLevel: 'loose',
      flowchart: {
        useMaxWidth: true,
        htmlLabels: true,
        curve: 'basis',
      },
    });

    const renderDiagram = async () => {
      if (!elementRef.current || !chart || chart.trim() === '') {
        setError('No diagram data provided');
        return;
      }

      try {
        setError(null);
        elementRef.current.innerHTML = '';
        
        // Clean the chart string - remove extra whitespace at start/end
        const cleanedChart = chart.trim();
        
        const { svg } = await mermaid.render(`${id}-${Date.now()}`, cleanedChart);
        elementRef.current.innerHTML = svg;
      } catch (err) {
        console.error('Mermaid rendering error:', err);
        setError('Failed to render diagram. Please check the diagram syntax.');
        
        // Show the raw diagram code for debugging
        if (elementRef.current) {
          elementRef.current.innerHTML = `
            <div class="p-4 bg-red-50 border border-red-200 rounded text-sm">
              <p class="text-red-700 font-semibold mb-2">Diagram Rendering Error</p>
              <pre class="text-xs overflow-auto">${cleanedChart}</pre>
            </div>
          `;
        }
      }
    };

    renderDiagram();
  }, [chart, id]);

  if (error && !chart) {
    return (
      <div className="p-4 bg-red-50 border border-red-200 rounded-lg text-red-700">
        {error}
      </div>
    );
  }

  return (
    <div 
      ref={elementRef} 
      className="mermaid-diagram w-full overflow-auto p-4 bg-white rounded-lg border"
    />
  );
}