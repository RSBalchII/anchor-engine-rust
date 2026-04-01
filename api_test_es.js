/*
 * Human UI Testing Script for Anchor Engine Rust
 * Tests the HTTP API endpoints to validate functionality
 */

import axios from 'axios';
import { writeFile } from 'fs/promises';

// Configuration
const BASE_URL = 'http://localhost:3160';
const TEST_DATA = {
  content: "Rust is a systems programming language focused on safety, speed, and concurrency. Its designers concentrated on a clean, well-documented type system, fearless concurrency, zero-cost abstractions, and minimal runtime.",
  filename: "rust_intro.md",
  query: "#rust #programming"
};

// Test results
let testResults = {
  passed: 0,
  failed: 0,
  tests: []
};

// Utility function to log test results
function logTest(name, success, details = "") {
  const result = { name, success, details };
  testResults.tests.push(result);
  
  if (success) {
    testResults.passed++;
    console.log(`✅ PASSED: ${name}`);
  } else {
    testResults.failed++;
    console.log(`❌ FAILED: ${name} - ${details}`);
  }
}

// Test health endpoint
async function testHealth() {
  try {
    const response = await axios.get(`${BASE_URL}/health`);
    logTest("Health Check", response.data.status === 'healthy', `Status: ${response.data.status}`);
  } catch (error) {
    logTest("Health Check", false, error.message);
  }
}

// Test ingest endpoint
async function testIngest() {
  try {
    const response = await axios.post(`${BASE_URL}/v1/memory/ingest`, {
      source: TEST_DATA.filename,
      content: TEST_DATA.content,
      bucket: "test"
    });
    
    logTest("Ingest Content", response.status === 200, `Status: ${response.status}`);
  } catch (error) {
    logTest("Ingest Content", false, error.message);
  }
}

// Test search endpoint
async function testSearch() {
  try {
    const response = await axios.post(`${BASE_URL}/v1/memory/search`, {
      query: TEST_DATA.query,
      max_chars: 5000
    });
    
    logTest("Search Functionality", response.status === 200, `Results: ${response.data.results?.length || 0}`);
  } catch (error) {
    logTest("Search Functionality", false, error.message);
  }
}

// Test stats endpoint
async function testStats() {
  try {
    const response = await axios.get(`${BASE_URL}/stats`);
    logTest("Stats Endpoint", response.status === 200, `Response: ${JSON.stringify(response.data)}`);
  } catch (error) {
    logTest("Stats Endpoint", false, error.message);
  }
}

// Run all tests
async function runTests() {
  console.log("🧪 Starting Human UI Testing for Anchor Engine Rust...\n");
  
  // Wait a moment for server to be ready
  await new Promise(resolve => setTimeout(resolve, 2000));
  
  await testHealth();
  await testIngest();
  await testSearch();
  await testStats();
  
  // Print summary
  console.log("\n📊 Test Results Summary:");
  console.log(`✅ Passed: ${testResults.passed}`);
  console.log(`❌ Failed: ${testResults.failed}`);
  console.log(`📈 Success Rate: ${Math.round((testResults.passed / testResults.tests.length) * 100)}%`);
  
  // Save detailed results
  await writeFile('api_test_results.json', JSON.stringify(testResults, null, 2));
  console.log("\n💾 Detailed results saved to api_test_results.json");
}

// Run the tests
runTests().catch(console.error);