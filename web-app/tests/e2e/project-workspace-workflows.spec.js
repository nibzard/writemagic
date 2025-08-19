/**
 * End-to-end tests for project workspace workflows
 * Tests multi-pane editing, project management, and workspace layouts
 */

const { test, expect } = require('@playwright/test');

test.describe('Project Workspace Workflows', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.waitForFunction(() => window.writemagic_wasm !== undefined);
    await expect(page.locator('.app-container')).toBeVisible();
  });

  test('should create and manage projects', async ({ page }) => {
    // Click on Projects tab
    await page.click('[data-testid="projects-tab"]');
    
    // Create new project
    await page.click('[data-testid="new-project-btn"]');
    await page.fill('[data-testid="project-name-input"]', 'My Novel Project');
    await page.fill('[data-testid="project-description-input"]', 'A science fiction novel about space exploration');
    await page.click('[data-testid="create-project-btn"]');
    
    // Verify project is created and listed
    await expect(page.locator('[data-testid="project-card"]')).toBeVisible();
    await expect(page.locator('[data-testid="project-title"]')).toHaveText('My Novel Project');
    await expect(page.locator('[data-testid="project-description"]')).toHaveText('A science fiction novel about space exploration');
    
    // Open project
    await page.click('[data-testid="open-project-btn"]');
    
    // Verify project workspace is loaded
    await expect(page.locator('[data-testid="project-workspace"]')).toBeVisible();
    await expect(page.locator('[data-testid="project-title-header"]')).toHaveText('My Novel Project');
    
    // Verify empty workspace state
    await expect(page.locator('[data-testid="empty-workspace"]')).toBeVisible();
  });

  test('should add documents to project and organize them', async ({ page }) => {
    // Create project
    await page.click('[data-testid="projects-tab"]');
    await page.click('[data-testid="new-project-btn"]');
    await page.fill('[data-testid="project-name-input"]', 'Research Project');
    await page.click('[data-testid="create-project-btn"]');
    await page.click('[data-testid="open-project-btn"]');
    
    // Add documents to project
    const documents = [
      { title: 'Introduction Chapter', content: 'This is the introduction to our research.' },
      { title: 'Literature Review', content: 'Review of existing literature in the field.' },
      { title: 'Methodology', content: 'Our research methodology and approach.' },
      { title: 'Results', content: 'Results and findings from our study.' }
    ];

    for (const doc of documents) {
      await page.click('[data-testid="add-document-to-project-btn"]');
      await page.fill('[data-testid="document-title-input"]', doc.title);
      await page.click('[data-testid="create-document-btn"]');
      
      // Add content
      await expect(page.locator('[data-testid="document-editor"]')).toBeVisible();
      await page.fill('[data-testid="document-content"]', doc.content);
      await expect(page.locator('[data-testid="save-status"]')).toHaveText('Saved');
      
      // Return to project view
      await page.click('[data-testid="back-to-project-btn"]');
    }
    
    // Verify all documents are listed in project
    await expect(page.locator('[data-testid="project-document-item"]')).toHaveCount(4);
    
    // Test document reordering
    const firstDoc = page.locator('[data-testid="project-document-item"]').first();
    const secondDoc = page.locator('[data-testid="project-document-item"]').nth(1);
    
    await firstDoc.dragTo(secondDoc);
    
    // Verify order changed
    await expect(page.locator('[data-testid="project-document-item"]').first()).toContainText('Literature Review');
  });

  test('should switch between different workspace layouts', async ({ page }) => {
    // Create project with documents
    await page.click('[data-testid="projects-tab"]');
    await page.click('[data-testid="new-project-btn"]');
    await page.fill('[data-testid="project-name-input"]', 'Layout Test Project');
    await page.click('[data-testid="create-project-btn"]');
    await page.click('[data-testid="open-project-btn"]');
    
    // Add two documents
    for (let i = 1; i <= 3; i++) {
      await page.click('[data-testid="add-document-to-project-btn"]');
      await page.fill('[data-testid="document-title-input"]', `Document ${i}`);
      await page.click('[data-testid="create-document-btn"]');
      await page.fill('[data-testid="document-content"]', `Content for document ${i}`);
      await expect(page.locator('[data-testid="save-status"]')).toHaveText('Saved');
      await page.click('[data-testid="back-to-project-btn"]');
    }
    
    // Test single pane layout (default)
    await expect(page.locator('[data-testid="editor-pane"]')).toHaveCount(1);
    
    // Switch to split layout
    await page.click('[data-testid="layout-controls"]');
    await page.click('[data-testid="split-layout-btn"]');
    
    await expect(page.locator('[data-testid="editor-pane"]')).toHaveCount(2);
    
    // Load documents into panes
    await page.locator('[data-testid="project-document-item"]').first().click();
    await expect(page.locator('[data-testid="editor-pane"]').first().locator('[data-testid="document-content"]')).toHaveValue('Content for document 1');
    
    // Click on second pane and load different document
    await page.click('[data-testid="editor-pane"]').nth(1);
    await page.locator('[data-testid="project-document-item"]').nth(1).click();
    await expect(page.locator('[data-testid="editor-pane"]').nth(1).locator('[data-testid="document-content"]')).toHaveValue('Content for document 2');
    
    // Switch to triple layout
    await page.click('[data-testid="layout-controls"]');
    await page.click('[data-testid="triple-layout-btn"]');
    
    await expect(page.locator('[data-testid="editor-pane"]')).toHaveCount(3);
    
    // Load third document in third pane
    await page.click('[data-testid="editor-pane"]').nth(2);
    await page.locator('[data-testid="project-document-item"]').nth(2).click();
    await expect(page.locator('[data-testid="editor-pane"]').nth(2).locator('[data-testid="document-content"]')).toHaveValue('Content for document 3');
    
    // Test quad layout
    await page.click('[data-testid="layout-controls"]');
    await page.click('[data-testid="quad-layout-btn"]');
    
    await expect(page.locator('[data-testid="editor-pane"]')).toHaveCount(4);
  });

  test('should enable concurrent editing across multiple panes', async ({ page }) => {
    // Set up project with split layout
    await page.click('[data-testid="projects-tab"]');
    await page.click('[data-testid="new-project-btn"]');
    await page.fill('[data-testid="project-name-input"]', 'Multi-Pane Editing');
    await page.click('[data-testid="create-project-btn"]');
    await page.click('[data-testid="open-project-btn"]');
    
    // Create two documents
    await page.click('[data-testid="add-document-to-project-btn"]');
    await page.fill('[data-testid="document-title-input"]', 'Chapter 1');
    await page.click('[data-testid="create-document-btn"]');
    await page.fill('[data-testid="document-content"]', 'The beginning of our story');
    await expect(page.locator('[data-testid="save-status"]')).toHaveText('Saved');
    await page.click('[data-testid="back-to-project-btn"]');
    
    await page.click('[data-testid="add-document-to-project-btn"]');
    await page.fill('[data-testid="document-title-input"]', 'Chapter 2');
    await page.click('[data-testid="create-document-btn"]');
    await page.fill('[data-testid="document-content"]', 'The story continues');
    await expect(page.locator('[data-testid="save-status"]')).toHaveText('Saved');
    await page.click('[data-testid="back-to-project-btn"]');
    
    // Switch to split layout
    await page.click('[data-testid="layout-controls"]');
    await page.click('[data-testid="split-layout-btn"]');
    
    // Load documents in both panes
    await page.locator('[data-testid="project-document-item"]').first().click();
    await page.click('[data-testid="editor-pane"]').nth(1);
    await page.locator('[data-testid="project-document-item"]').nth(1).click();
    
    // Edit both documents simultaneously
    const leftPane = page.locator('[data-testid="editor-pane"]').first();
    const rightPane = page.locator('[data-testid="editor-pane"]').nth(1);
    
    await leftPane.locator('[data-testid="document-content"]').fill('The beginning of our story. Our hero awakens.');
    await rightPane.locator('[data-testid="document-content"]').fill('The story continues. The adventure begins.');
    
    // Verify both documents are saved
    await expect(leftPane.locator('[data-testid="save-status"]')).toHaveText('Saved');
    await expect(rightPane.locator('[data-testid="save-status"]')).toHaveText('Saved');
    
    // Test copying content between panes
    await leftPane.locator('[data-testid="document-content"]').selectText();
    await page.keyboard.press('Control+C');
    
    await rightPane.locator('[data-testid="document-content"]').click();
    await page.keyboard.press('Control+A');
    await page.keyboard.press('Control+V');
    
    await expect(rightPane.locator('[data-testid="document-content"]')).toHaveValue('The beginning of our story. Our hero awakens.');
  });

  test('should manage project settings and metadata', async ({ page }) => {
    // Create project
    await page.click('[data-testid="projects-tab"]');
    await page.click('[data-testid="new-project-btn"]');
    await page.fill('[data-testid="project-name-input"]', 'Settings Test Project');
    await page.fill('[data-testid="project-description-input"]', 'Testing project settings');
    await page.click('[data-testid="create-project-btn"]');
    await page.click('[data-testid="open-project-btn"]');
    
    // Open project settings
    await page.click('[data-testid="project-menu-btn"]');
    await page.click('[data-testid="project-settings-btn"]');
    
    await expect(page.locator('[data-testid="project-settings-modal"]')).toBeVisible();
    
    // Update project details
    await page.fill('[data-testid="settings-project-name"]', 'Updated Project Name');
    await page.fill('[data-testid="settings-project-description"]', 'Updated project description with more details');
    
    // Set project tags
    await page.fill('[data-testid="project-tags-input"]', 'fiction, novel, draft');
    await page.press('[data-testid="project-tags-input"]', 'Enter');
    
    // Set word count goal
    await page.fill('[data-testid="project-word-goal"]', '50000');
    
    // Set project status
    await page.selectOption('[data-testid="project-status-select"]', 'in-progress');
    
    // Save settings
    await page.click('[data-testid="save-project-settings-btn"]');
    
    await expect(page.locator('[data-testid="project-settings-modal"]')).not.toBeVisible();
    
    // Verify settings are applied
    await expect(page.locator('[data-testid="project-title-header"]')).toHaveText('Updated Project Name');
    
    // Check project metadata display
    await expect(page.locator('[data-testid="project-status"]')).toHaveText('In Progress');
    await expect(page.locator('[data-testid="project-word-goal"]')).toHaveText('Goal: 50,000 words');
    await expect(page.locator('[data-testid="project-tags"]')).toContainText('fiction');
    await expect(page.locator('[data-testid="project-tags"]')).toContainText('novel');
    await expect(page.locator('[data-testid="project-tags"]')).toContainText('draft');
  });

  test('should track project progress and statistics', async ({ page }) => {
    // Create project with content
    await page.click('[data-testid="projects-tab"]');
    await page.click('[data-testid="new-project-btn"]');
    await page.fill('[data-testid="project-name-input"]', 'Progress Tracking Project');
    await page.click('[data-testid="create-project-btn"]');
    await page.click('[data-testid="open-project-btn"]');
    
    // Set word goal
    await page.click('[data-testid="project-menu-btn"]');
    await page.click('[data-testid="project-settings-btn"]');
    await page.fill('[data-testid="project-word-goal"]', '1000');
    await page.click('[data-testid="save-project-settings-btn"]');
    
    // Add documents with content
    const documents = [
      { title: 'Document 1', content: 'Lorem ipsum dolor sit amet consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam quis nostrud exercitation.' },
      { title: 'Document 2', content: 'Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident sunt in culpa qui officia deserunt mollit anim.' }
    ];

    for (const doc of documents) {
      await page.click('[data-testid="add-document-to-project-btn"]');
      await page.fill('[data-testid="document-title-input"]', doc.title);
      await page.click('[data-testid="create-document-btn"]');
      await page.fill('[data-testid="document-content"]', doc.content);
      await expect(page.locator('[data-testid="save-status"]')).toHaveText('Saved');
      await page.click('[data-testid="back-to-project-btn"]');
    }
    
    // Open project statistics
    await page.click('[data-testid="project-menu-btn"]');
    await page.click('[data-testid="project-statistics-btn"]');
    
    await expect(page.locator('[data-testid="project-statistics-panel"]')).toBeVisible();
    
    // Verify statistics
    const totalWords = await page.textContent('[data-testid="total-word-count"]');
    expect(parseInt(totalWords)).toBeGreaterThan(50);
    
    const documentCount = await page.textContent('[data-testid="document-count"]');
    expect(parseInt(documentCount)).toBe(2);
    
    // Check progress toward goal
    const progressBar = page.locator('[data-testid="progress-bar"]');
    await expect(progressBar).toBeVisible();
    
    const progressText = await page.textContent('[data-testid="progress-text"]');
    expect(progressText).toMatch(/\d+\/1000 words/);
    
    // Test daily writing statistics
    await page.click('[data-testid="daily-stats-tab"]');
    
    await expect(page.locator('[data-testid="daily-stats-chart"]')).toBeVisible();
    await expect(page.locator('[data-testid="words-today"]')).toBeVisible();
    
    // Test weekly/monthly views
    await page.click('[data-testid="weekly-view-btn"]');
    await expect(page.locator('[data-testid="weekly-stats-chart"]')).toBeVisible();
    
    await page.click('[data-testid="monthly-view-btn"]');
    await expect(page.locator('[data-testid="monthly-stats-chart"]')).toBeVisible();
  });

  test('should export entire project', async ({ page }) => {
    // Create project with multiple documents
    await page.click('[data-testid="projects-tab"]');
    await page.click('[data-testid="new-project-btn"]');
    await page.fill('[data-testid="project-name-input"]', 'Export Test Project');
    await page.click('[data-testid="create-project-btn"]');
    await page.click('[data-testid="open-project-btn"]');
    
    // Add documents
    const documents = [
      { title: 'Chapter 1', content: '# Chapter 1\n\nThis is the first chapter.' },
      { title: 'Chapter 2', content: '# Chapter 2\n\nThis is the second chapter.' },
      { title: 'Appendix', content: '# Appendix\n\nAdditional information.' }
    ];

    for (const doc of documents) {
      await page.click('[data-testid="add-document-to-project-btn"]');
      await page.fill('[data-testid="document-title-input"]', doc.title);
      await page.click('[data-testid="create-document-btn"]');
      await page.fill('[data-testid="document-content"]', doc.content);
      await expect(page.locator('[data-testid="save-status"]')).toHaveText('Saved');
      await page.click('[data-testid="back-to-project-btn"]');
    }
    
    // Export project
    await page.click('[data-testid="project-menu-btn"]');
    await page.click('[data-testid="export-project-btn"]');
    
    await expect(page.locator('[data-testid="export-options-modal"]')).toBeVisible();
    
    // Test different export options
    
    // Export as combined document
    const [combinedDownload] = await Promise.all([
      page.waitForEvent('download'),
      page.click('[data-testid="export-combined-btn"]')
    ]);
    
    expect(combinedDownload.suggestedFilename()).toContain('Export Test Project');
    expect(combinedDownload.suggestedFilename()).toContain('.md');
    
    // Export as ZIP archive
    const [zipDownload] = await Promise.all([
      page.waitForEvent('download'),
      page.click('[data-testid="export-zip-btn"]')
    ]);
    
    expect(zipDownload.suggestedFilename()).toContain('Export Test Project');
    expect(zipDownload.suggestedFilename()).toContain('.zip');
    
    // Export with custom format
    await page.selectOption('[data-testid="export-format-select"]', 'html');
    await page.check('[data-testid="include-toc-checkbox"]');
    await page.check('[data-testid="include-metadata-checkbox"]');
    
    const [customDownload] = await Promise.all([
      page.waitForEvent('download'),
      page.click('[data-testid="export-custom-btn"]')
    ]);
    
    expect(customDownload.suggestedFilename()).toContain('.html');
  });

  test('should handle project collaboration features', async ({ page }) => {
    // Create project
    await page.click('[data-testid="projects-tab"]');
    await page.click('[data-testid="new-project-btn"]');
    await page.fill('[data-testid="project-name-input"]', 'Collaboration Project');
    await page.click('[data-testid="create-project-btn"]');
    await page.click('[data-testid="open-project-btn"]');
    
    // Open collaboration settings
    await page.click('[data-testid="project-menu-btn"]');
    await page.click('[data-testid="collaboration-settings-btn"]');
    
    await expect(page.locator('[data-testid="collaboration-panel"]')).toBeVisible();
    
    // Enable collaboration
    await page.check('[data-testid="enable-collaboration-checkbox"]');
    
    // Generate share link
    await page.click('[data-testid="generate-share-link-btn"]');
    
    await expect(page.locator('[data-testid="share-link"]')).toBeVisible();
    
    // Copy share link
    await page.click('[data-testid="copy-share-link-btn"]');
    
    // Verify copy success feedback
    await expect(page.locator('[data-testid="copy-success-message"]')).toBeVisible();
    
    // Add collaborator by email
    await page.fill('[data-testid="collaborator-email-input"]', 'alice@example.com');
    await page.selectOption('[data-testid="collaborator-role-select"]', 'editor');
    await page.click('[data-testid="add-collaborator-btn"]');
    
    // Verify collaborator is added
    await expect(page.locator('[data-testid="collaborator-item"]')).toBeVisible();
    await expect(page.locator('[data-testid="collaborator-email"]')).toHaveText('alice@example.com');
    await expect(page.locator('[data-testid="collaborator-role"]')).toHaveText('Editor');
    
    // Test permission changes
    await page.click('[data-testid="collaborator-role-dropdown"]');
    await page.click('[data-testid="change-to-viewer-btn"]');
    
    await expect(page.locator('[data-testid="collaborator-role"]')).toHaveText('Viewer');
    
    // Remove collaborator
    await page.click('[data-testid="remove-collaborator-btn"]');
    await page.click('[data-testid="confirm-remove-btn"]');
    
    await expect(page.locator('[data-testid="collaborator-item"]')).not.toBeVisible();
    
    // Test activity feed
    await page.click('[data-testid="project-activity-tab"]');
    
    await expect(page.locator('[data-testid="activity-feed"]')).toBeVisible();
    await expect(page.locator('[data-testid="activity-item"]')).toHaveCount.toBeGreaterThan(0);
  });

  test('should handle workspace persistence and restoration', async ({ page }) => {
    // Create complex workspace setup
    await page.click('[data-testid="projects-tab"]');
    await page.click('[data-testid="new-project-btn"]');
    await page.fill('[data-testid="project-name-input"]', 'Persistence Test');
    await page.click('[data-testid="create-project-btn"]');
    await page.click('[data-testid="open-project-btn"]');
    
    // Add documents
    for (let i = 1; i <= 3; i++) {
      await page.click('[data-testid="add-document-to-project-btn"]');
      await page.fill('[data-testid="document-title-input"]', `Document ${i}`);
      await page.click('[data-testid="create-document-btn"]');
      await page.fill('[data-testid="document-content"]', `Content ${i}`);
      await expect(page.locator('[data-testid="save-status"]')).toHaveText('Saved');
      await page.click('[data-testid="back-to-project-btn"]');
    }
    
    // Set up triple layout with documents
    await page.click('[data-testid="layout-controls"]');
    await page.click('[data-testid="triple-layout-btn"]');
    
    await page.locator('[data-testid="project-document-item"]').first().click();
    await page.click('[data-testid="editor-pane"]').nth(1);
    await page.locator('[data-testid="project-document-item"]').nth(1).click();
    await page.click('[data-testid="editor-pane"]').nth(2);
    await page.locator('[data-testid="project-document-item"]').nth(2).click();
    
    // Reload page to test persistence
    await page.reload();
    await page.waitForLoadState('networkidle');
    await page.waitForFunction(() => window.writemagic_wasm !== undefined);
    
    // Navigate back to project
    await page.click('[data-testid="projects-tab"]');
    await page.locator('[data-testid="project-card"]').first().click();
    await page.click('[data-testid="open-project-btn"]');
    
    // Verify workspace state is restored
    await expect(page.locator('[data-testid="editor-pane"]')).toHaveCount(3);
    await expect(page.locator('[data-testid="editor-pane"]').first().locator('[data-testid="document-content"]')).toHaveValue('Content 1');
    await expect(page.locator('[data-testid="editor-pane"]').nth(1).locator('[data-testid="document-content"]')).toHaveValue('Content 2');
    await expect(page.locator('[data-testid="editor-pane"]').nth(2).locator('[data-testid="document-content"]')).toHaveValue('Content 3');
  });
});