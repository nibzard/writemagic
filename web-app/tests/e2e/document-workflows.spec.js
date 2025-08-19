/**
 * End-to-end tests for document management workflows
 * Tests complete user journeys from UI interaction to data persistence
 */

const { test, expect } = require('@playwright/test');

test.describe('Document Management Workflows', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for WASM module to load
    await page.waitForFunction(() => window.writemagic_wasm !== undefined);
    
    // Ensure the app is fully initialized
    await expect(page.locator('.app-container')).toBeVisible();
  });

  test('should create and edit a new document', async ({ page }) => {
    // Click on "New Document" button
    await page.click('[data-testid="new-document-btn"]');
    
    // Fill in document title
    await page.fill('[data-testid="document-title-input"]', 'My First Document');
    
    // Click create button
    await page.click('[data-testid="create-document-btn"]');
    
    // Wait for document to be created and editor to appear
    await expect(page.locator('[data-testid="document-editor"]')).toBeVisible();
    
    // Verify document title is displayed
    await expect(page.locator('[data-testid="document-title"]')).toHaveText('My First Document');
    
    // Type content in the editor
    const editorContent = 'This is my first document content. It contains multiple sentences.';
    await page.fill('[data-testid="document-content"]', editorContent);
    
    // Wait for auto-save indicator
    await expect(page.locator('[data-testid="save-status"]')).toHaveText('Saved');
    
    // Verify word count is updated
    await expect(page.locator('[data-testid="word-count"]')).toHaveText('10 words');
    
    // Verify character count
    await expect(page.locator('[data-testid="char-count"]')).toHaveText(`${editorContent.length} characters`);
  });

  test('should auto-save document changes', async ({ page }) => {
    // Create a new document
    await page.click('[data-testid="new-document-btn"]');
    await page.fill('[data-testid="document-title-input"]', 'Auto-save Test Document');
    await page.click('[data-testid="create-document-btn"]');
    
    // Wait for editor
    await expect(page.locator('[data-testid="document-editor"]')).toBeVisible();
    
    // Type content
    await page.fill('[data-testid="document-content"]', 'Testing auto-save functionality');
    
    // Wait for save status to change to "Saving..."
    await expect(page.locator('[data-testid="save-status"]')).toHaveText('Saving...');
    
    // Then wait for "Saved"
    await expect(page.locator('[data-testid="save-status"]')).toHaveText('Saved', { timeout: 5000 });
    
    // Refresh page to verify persistence
    await page.reload();
    await page.waitForLoadState('networkidle');
    
    // Check if document content is restored
    await expect(page.locator('[data-testid="document-content"]')).toHaveValue('Testing auto-save functionality');
  });

  test('should delete a document', async ({ page }) => {
    // Create a document first
    await page.click('[data-testid="new-document-btn"]');
    await page.fill('[data-testid="document-title-input"]', 'Document to Delete');
    await page.click('[data-testid="create-document-btn"]');
    
    await expect(page.locator('[data-testid="document-editor"]')).toBeVisible();
    
    // Go to document list
    await page.click('[data-testid="documents-tab"]');
    
    // Find the document in the list
    const documentRow = page.locator('[data-testid="document-row"]').filter({ hasText: 'Document to Delete' });
    await expect(documentRow).toBeVisible();
    
    // Click delete button
    await documentRow.locator('[data-testid="delete-document-btn"]').click();
    
    // Confirm deletion in modal
    await page.click('[data-testid="confirm-delete-btn"]');
    
    // Verify document is removed from list
    await expect(documentRow).not.toBeVisible();
    
    // Verify empty state is shown if no documents remain
    const documentsList = page.locator('[data-testid="documents-list"]');
    if (await documentsList.locator('[data-testid="document-row"]').count() === 0) {
      await expect(page.locator('[data-testid="empty-documents"]')).toBeVisible();
    }
  });

  test('should search and filter documents', async ({ page }) => {
    // Create multiple documents with different content
    const documents = [
      { title: 'React Tutorial', content: 'Learn React hooks and components' },
      { title: 'JavaScript Basics', content: 'Variables, functions, and objects' },
      { title: 'CSS Grid Guide', content: 'Modern layout techniques with CSS Grid' }
    ];

    for (const doc of documents) {
      await page.click('[data-testid="new-document-btn"]');
      await page.fill('[data-testid="document-title-input"]', doc.title);
      await page.click('[data-testid="create-document-btn"]');
      
      await expect(page.locator('[data-testid="document-editor"]')).toBeVisible();
      await page.fill('[data-testid="document-content"]', doc.content);
      await expect(page.locator('[data-testid="save-status"]')).toHaveText('Saved');
      
      // Navigate back to create next document
      await page.click('[data-testid="documents-tab"]');
    }
    
    // Test search functionality
    await page.fill('[data-testid="document-search"]', 'React');
    
    // Should show only React-related document
    await expect(page.locator('[data-testid="document-row"]')).toHaveCount(1);
    await expect(page.locator('[data-testid="document-row"]')).toContainText('React Tutorial');
    
    // Clear search and test content search
    await page.fill('[data-testid="document-search"]', '');
    await expect(page.locator('[data-testid="document-row"]')).toHaveCount(3);
    
    // Search by content
    await page.fill('[data-testid="document-search"]', 'CSS');
    await expect(page.locator('[data-testid="document-row"]')).toHaveCount(1);
    await expect(page.locator('[data-testid="document-row"]')).toContainText('CSS Grid Guide');
  });

  test('should handle document versioning and history', async ({ page }) => {
    // Create a document
    await page.click('[data-testid="new-document-btn"]');
    await page.fill('[data-testid="document-title-input"]', 'Version Test Document');
    await page.click('[data-testid="create-document-btn"]');
    
    await expect(page.locator('[data-testid="document-editor"]')).toBeVisible();
    
    // Add initial content
    await page.fill('[data-testid="document-content"]', 'Version 1 content');
    await expect(page.locator('[data-testid="save-status"]')).toHaveText('Saved');
    
    // Make significant changes
    await page.fill('[data-testid="document-content"]', 'Version 1 content\n\nVersion 2 additions with more content');
    await expect(page.locator('[data-testid="save-status"]')).toHaveText('Saved');
    
    // Open version history
    await page.click('[data-testid="document-menu-btn"]');
    await page.click('[data-testid="view-history-btn"]');
    
    // Verify version history panel
    await expect(page.locator('[data-testid="version-history"]')).toBeVisible();
    
    // Should show multiple versions
    const versionItems = page.locator('[data-testid="version-item"]');
    await expect(versionItems).toHaveCount(2);
    
    // Click on first version to preview
    await versionItems.first().click();
    
    // Should show preview of old version
    await expect(page.locator('[data-testid="version-preview"]')).toContainText('Version 1 content');
    await expect(page.locator('[data-testid="version-preview"]')).not.toContainText('Version 2 additions');
    
    // Restore to previous version
    await page.click('[data-testid="restore-version-btn"]');
    await page.click('[data-testid="confirm-restore-btn"]');
    
    // Verify content is restored
    await expect(page.locator('[data-testid="document-content"]')).toHaveValue('Version 1 content');
  });

  test('should export document in different formats', async ({ page }) => {
    // Create a document with rich content
    await page.click('[data-testid="new-document-btn"]');
    await page.fill('[data-testid="document-title-input"]', 'Export Test Document');
    await page.click('[data-testid="create-document-btn"]');
    
    const content = `# Export Test Document

This is a **bold** text and this is *italic*.

## List Example
- Item 1
- Item 2
- Item 3

> This is a blockquote

Code example:
\`\`\`javascript
console.log("Hello, World!");
\`\`\``;

    await page.fill('[data-testid="document-content"]', content);
    await expect(page.locator('[data-testid="save-status"]')).toHaveText('Saved');
    
    // Test different export formats
    await page.click('[data-testid="document-menu-btn"]');
    await page.click('[data-testid="export-btn"]');
    
    // Export as Markdown
    const [markdownDownload] = await Promise.all([
      page.waitForEvent('download'),
      page.click('[data-testid="export-markdown-btn"]')
    ]);
    
    expect(markdownDownload.suggestedFilename()).toContain('Export Test Document');
    expect(markdownDownload.suggestedFilename()).toContain('.md');
    
    // Export as Plain Text
    const [textDownload] = await Promise.all([
      page.waitForEvent('download'),
      page.click('[data-testid="export-text-btn"]')
    ]);
    
    expect(textDownload.suggestedFilename()).toContain('Export Test Document');
    expect(textDownload.suggestedFilename()).toContain('.txt');
    
    // Export as HTML
    const [htmlDownload] = await Promise.all([
      page.waitForEvent('download'),
      page.click('[data-testid="export-html-btn"]')
    ]);
    
    expect(htmlDownload.suggestedFilename()).toContain('Export Test Document');
    expect(htmlDownload.suggestedFilename()).toContain('.html');
  });

  test('should handle collaborative editing indicators', async ({ page }) => {
    // Create a document
    await page.click('[data-testid="new-document-btn"]');
    await page.fill('[data-testid="document-title-input"]', 'Collaboration Test');
    await page.click('[data-testid="create-document-btn"]');
    
    await expect(page.locator('[data-testid="document-editor"]')).toBeVisible();
    
    // Simulate another user joining (this would normally come from WebSocket)
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('userJoined', {
        detail: {
          userId: 'user-2',
          userName: 'Alice Smith',
          cursor: { line: 0, column: 0 }
        }
      }));
    });
    
    // Should show collaboration indicator
    await expect(page.locator('[data-testid="collaborators-indicator"]')).toBeVisible();
    await expect(page.locator('[data-testid="collaborator-avatar"]')).toHaveCount(1);
    
    // Show collaborator cursor
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('cursorMoved', {
        detail: {
          userId: 'user-2',
          cursor: { line: 1, column: 10 }
        }
      }));
    });
    
    await expect(page.locator('[data-testid="remote-cursor"]')).toBeVisible();
    
    // Simulate user leaving
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('userLeft', {
        detail: { userId: 'user-2' }
      }));
    });
    
    await expect(page.locator('[data-testid="collaborator-avatar"]')).toHaveCount(0);
  });

  test('should handle document templates', async ({ page }) => {
    // Open new document with template option
    await page.click('[data-testid="new-document-btn"]');
    await page.click('[data-testid="use-template-btn"]');
    
    // Select a template
    await expect(page.locator('[data-testid="template-gallery"]')).toBeVisible();
    await page.click('[data-testid="template-blog-post"]');
    
    // Fill in template details
    await page.fill('[data-testid="document-title-input"]', 'My Blog Post');
    await page.click('[data-testid="create-from-template-btn"]');
    
    // Verify template content is loaded
    await expect(page.locator('[data-testid="document-editor"]')).toBeVisible();
    const contentValue = await page.inputValue('[data-testid="document-content"]');
    
    expect(contentValue).toContain('# My Blog Post');
    expect(contentValue).toContain('## Introduction');
    expect(contentValue).toContain('## Main Content');
    expect(contentValue).toContain('## Conclusion');
    
    // Customize the template
    await page.fill('[data-testid="document-content"]', contentValue + '\n\nCustom content added to template');
    await expect(page.locator('[data-testid="save-status"]')).toHaveText('Saved');
  });

  test('should validate document content and show statistics', async ({ page }) => {
    // Create a document with substantial content
    await page.click('[data-testid="new-document-btn"]');
    await page.fill('[data-testid="document-title-input"]', 'Statistics Test');
    await page.click('[data-testid="create-document-btn"]');
    
    const content = `Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

This document contains multiple paragraphs with various sentence structures. Some sentences are short. Others are longer and contain more complex grammatical structures with subordinate clauses.`;

    await page.fill('[data-testid="document-content"]', content);
    
    // Open statistics panel
    await page.click('[data-testid="document-menu-btn"]');
    await page.click('[data-testid="view-statistics-btn"]');
    
    await expect(page.locator('[data-testid="document-statistics"]')).toBeVisible();
    
    // Verify statistics are calculated correctly
    const wordCount = await page.textContent('[data-testid="stat-word-count"]');
    expect(parseInt(wordCount)).toBeGreaterThan(70);
    
    const charCount = await page.textContent('[data-testid="stat-char-count"]');
    expect(parseInt(charCount)).toBe(content.length);
    
    const paragraphCount = await page.textContent('[data-testid="stat-paragraph-count"]');
    expect(parseInt(paragraphCount)).toBe(3);
    
    const sentenceCount = await page.textContent('[data-testid="stat-sentence-count"]');
    expect(parseInt(sentenceCount)).toBeGreaterThan(5);
    
    // Check readability score
    await expect(page.locator('[data-testid="readability-score"]')).toBeVisible();
    
    // Test reading time estimate
    const readingTime = await page.textContent('[data-testid="reading-time"]');
    expect(readingTime).toMatch(/\d+ min read/);
  });
});