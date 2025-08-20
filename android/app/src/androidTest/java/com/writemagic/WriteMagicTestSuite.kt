package com.writemagic

import com.writemagic.e2e.EndToEndWorkflowTest
import com.writemagic.integration.DocumentManagementTest
import com.writemagic.integration.FFIIntegrationTest
import com.writemagic.ui.AIIntegrationTest
import com.writemagic.ui.NavigationTest
import com.writemagic.ui.WritingWorkflowTest
import org.junit.runner.RunWith
import org.junit.runners.Suite

/**
 * Test suite for running all WriteMagic Android tests.
 * Organizes tests by category and execution order.
 */
@RunWith(Suite::class)
@Suite.SuiteClasses(
    // Unit-level integration tests
    FFIIntegrationTest::class,
    
    // UI component tests
    NavigationTest::class,
    AIIntegrationTest::class,
    WritingWorkflowTest::class,
    
    // Document workflow tests
    DocumentManagementTest::class,
    
    // End-to-end workflow tests
    EndToEndWorkflowTest::class
)
class WriteMagicTestSuite

/**
 * Smoke test suite for quick validation
 */
@RunWith(Suite::class)
@Suite.SuiteClasses(
    NavigationTest::class,
    WritingWorkflowTest::class
)
class WriteMagicSmokeTestSuite

/**
 * Performance test suite for load testing
 */
@RunWith(Suite::class)
@Suite.SuiteClasses(
    FFIIntegrationTest::class,
    DocumentManagementTest::class
)
class WriteMagicPerformanceTestSuite