/*
 * Simple connectivity test for Anchor Engine Rust
 * Tests basic server connectivity
 */

// Test if the server is running
async function testServer() {
  try {
    // Using fetch to test connectivity
    const response = await fetch('http://localhost:3160/health');
    const data = await response.json();
    
    console.log('✅ Server is running!');
    console.log('Health status:', data.status);
    console.log('Timestamp:', data.timestamp);
    console.log('Message:', data.message || data.health_message);
  } catch (error) {
    console.log('❌ Server is not accessible');
    console.log('Error:', error.message);
    console.log('');
    console.log('💡 Make sure to start the server first:');
    console.log('   cargo run --bin anchor-engine -- --port 3160');
  }
}

testServer();