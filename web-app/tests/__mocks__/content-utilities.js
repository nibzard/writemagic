// Mock ContentUtilities for tests
export class ContentUtilities {
  applyTemplate(template, content) {
    return content;
  }

  validateContent(content) {
    return { isValid: true, errors: [] };
  }

  countWords(text) {
    if (!text) return 0;
    return text.split(/\s+/).filter(word => word.length > 0).length;
  }

  analyzeChanges(oldContent, newContent) {
    return { added: 0, removed: 0, modified: 0 };
  }

  searchDocuments(documents, query) {
    return documents.filter(doc => 
      doc.title.includes(query) || doc.content.includes(query)
    );
  }

  estimateReadingTime(content) {
    return 5; // minutes
  }

  analyzeComplexity(content) {
    return 'medium';
  }
}

export default ContentUtilities;