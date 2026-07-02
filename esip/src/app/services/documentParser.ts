// Document Parser - Extract text from various file formats

import * as XLSX from 'xlsx';

export interface ParsedDocument {
  text: string;
  fileName: string;
  fileType: string;
  pageCount?: number;
  metadata?: Record<string, any>;
}

// Parse PDF files
async function parsePDF(file: File): Promise<ParsedDocument> {
  try {
    const pdfjsLib = await import('pdfjs-dist');
    
    // Set worker from CDN
    pdfjsLib.GlobalWorkerOptions.workerSrc = `https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.worker.min.js`;
    
    const arrayBuffer = await file.arrayBuffer();
    const pdf = await pdfjsLib.getDocument({ data: arrayBuffer }).promise;
    
    let fullText = '';
    const pageCount = pdf.numPages;
    
    for (let i = 1; i <= pageCount; i++) {
      const page = await pdf.getPage(i);
      const textContent = await page.getTextContent();
      const pageText = textContent.items
        .map((item: any) => item.str)
        .join(' ');
      fullText += pageText + '\n\n';
    }
    
    return {
      text: fullText.trim(),
      fileName: file.name,
      fileType: 'pdf',
      pageCount,
    };
  } catch (error) {
    console.error('PDF parsing error:', error);
    throw new Error('Failed to parse PDF file. Please try another format.');
  }
}

// Parse DOCX files
async function parseDOCX(file: File): Promise<ParsedDocument> {
  try {
    const mammoth = await import('mammoth');
    const arrayBuffer = await file.arrayBuffer();
    const result = await mammoth.extractRawText({ arrayBuffer });
    
    return {
      text: result.value.trim(),
      fileName: file.name,
      fileType: 'docx',
    };
  } catch (error) {
    console.error('DOCX parsing error:', error);
    throw new Error('Failed to parse DOCX file. Please try another format.');
  }
}

// Parse DOC files (older Word format)
async function parseDOC(file: File): Promise<ParsedDocument> {
  // For older .doc files, we'll try mammoth (it may work for some)
  // In production, you might need a dedicated library or backend service
  try {
    return await parseDOCX(file);
  } catch (error) {
    throw new Error('Legacy .doc format not fully supported. Please convert to .docx or PDF.');
  }
}

// Parse Excel files
async function parseExcel(file: File): Promise<ParsedDocument> {
  const arrayBuffer = await file.arrayBuffer();
  const workbook = XLSX.read(arrayBuffer, { type: 'array' });
  
  let fullText = '';
  
  // Parse all sheets
  workbook.SheetNames.forEach((sheetName) => {
    const worksheet = workbook.Sheets[sheetName];
    const sheetData = XLSX.utils.sheet_to_json(worksheet, { header: 1 });
    
    fullText += `\n\n=== Sheet: ${sheetName} ===\n\n`;
    
    sheetData.forEach((row: any) => {
      if (Array.isArray(row) && row.length > 0) {
        fullText += row.join(' | ') + '\n';
      }
    });
  });
  
  return {
    text: fullText.trim(),
    fileName: file.name,
    fileType: file.name.endsWith('.xlsx') ? 'xlsx' : 'xls',
    metadata: {
      sheetCount: workbook.SheetNames.length,
      sheetNames: workbook.SheetNames,
    },
  };
}

// Main parser function
export async function parseDocument(file: File): Promise<ParsedDocument> {
  const fileName = file.name.toLowerCase();
  
  try {
    if (fileName.endsWith('.pdf')) {
      return await parsePDF(file);
    } else if (fileName.endsWith('.docx')) {
      return await parseDOCX(file);
    } else if (fileName.endsWith('.doc')) {
      return await parseDOC(file);
    } else if (fileName.endsWith('.xlsx') || fileName.endsWith('.xls')) {
      return await parseExcel(file);
    } else {
      throw new Error('Unsupported file format. Please upload PDF, DOC, DOCX, XLS, or XLSX files.');
    }
  } catch (error) {
    console.error('Document parsing error:', error);
    throw error;
  }
}

// Validate file before parsing
export function validateFile(file: File): { valid: boolean; error?: string } {
  const maxSize = 10 * 1024 * 1024; // 10MB
  const allowedTypes = [
    'application/pdf',
    'application/vnd.openxmlformats-officedocument.wordprocessingml.document', // .docx
    'application/msword', // .doc
    'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet', // .xlsx
    'application/vnd.ms-excel', // .xls
  ];
  
  const fileName = file.name.toLowerCase();
  const allowedExtensions = ['.pdf', '.doc', '.docx', '.xls', '.xlsx'];
  
  if (file.size > maxSize) {
    return { valid: false, error: 'File size exceeds 10MB limit' };
  }
  
  const hasValidExtension = allowedExtensions.some(ext => fileName.endsWith(ext));
  if (!hasValidExtension) {
    return { valid: false, error: 'Invalid file type. Allowed: PDF, DOC, DOCX, XLS, XLSX' };
  }
  
  return { valid: true };
}