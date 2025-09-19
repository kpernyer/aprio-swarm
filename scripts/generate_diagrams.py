#!/usr/bin/env python3
"""
Generate Mermaid diagrams as images for the Living Twin Swarm System
"""

import os
import subprocess
import sys

def check_mermaid_cli():
    """Check if mermaid CLI is installed"""
    try:
        result = subprocess.run(['mmdc', '--version'], capture_output=True, text=True)
        return result.returncode == 0
    except FileNotFoundError:
        return False

def install_mermaid_cli():
    """Install mermaid CLI if not available"""
    print("📦 Installing Mermaid CLI...")
    try:
        subprocess.run(['npm', 'install', '-g', '@mermaid-js/mermaid-cli'], check=True)
        print("✅ Mermaid CLI installed successfully!")
        return True
    except subprocess.CalledProcessError:
        print("❌ Failed to install Mermaid CLI. Please install Node.js and npm first.")
        return False

def generate_diagram(mermaid_code, output_file, title):
    """Generate a diagram from Mermaid code"""
    print(f"🎨 Generating {title}...")
    
    # Create temporary mermaid file
    temp_file = f"temp_{title.lower().replace(' ', '_')}.mmd"
    with open(temp_file, 'w') as f:
        f.write(mermaid_code)
    
    try:
        # Generate the diagram
        subprocess.run([
            'mmdc', 
            '-i', temp_file, 
            '-o', output_file,
            '-t', 'default',
            '-b', 'white'
        ], check=True)
        
        print(f"✅ {title} saved as {output_file}")
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ Failed to generate {title}: {e}")
        return False
    finally:
        # Clean up temp file
        if os.path.exists(temp_file):
            os.remove(temp_file)

def main():
    """Main function to generate all diagrams"""
    print("🚀 Aprio Swarm System - Diagram Generator")
    print("=" * 50)
    
    # Check if mermaid CLI is available
    if not check_mermaid_cli():
        print("⚠️  Mermaid CLI not found. Installing...")
        if not install_mermaid_cli():
            print("\n📋 Manual Installation Instructions:")
            print("1. Install Node.js: https://nodejs.org/")
            print("2. Run: npm install -g @mermaid-js/mermaid-cli")
            print("3. Run this script again")
            return
    
    # Create output directory
    os.makedirs('docs/images', exist_ok=True)
    
    # System Flow Diagram
    system_flow = """
graph TB
    subgraph "Client Layer"
        A[Task Submission] --> B[SwarmCoordinator]
    end
    
    subgraph "Coordination Layer"
        B --> C[Task Queue]
        B --> D[Worker Registry]
        B --> E[Result Processor]
    end
    
    subgraph "Worker Pool"
        F[Document Processor<br/>Worker 1]
        G[ML Inference<br/>Worker 2]
        H[Vector Indexer<br/>Worker N]
    end
    
    subgraph "Task Processing"
        I[Echo Tasks<br/>100ms]
        J[Compute Tasks<br/>200ms]
        K[Error Tasks<br/>50ms]
    end
    
    C --> F
    C --> G
    C --> H
    
    F --> I
    G --> J
    H --> K
    
    I --> E
    J --> E
    K --> E
    
    E --> L[Task Results]
    
    style A fill:#e1f5fe
    style B fill:#f3e5f5
    style F fill:#e8f5e8
    style G fill:#e8f5e8
    style H fill:#e8f5e8
    style L fill:#fff3e0
    """
    
    # Task Flow Sequence
    task_flow = """
sequenceDiagram
    participant Client
    participant Coordinator
    participant Worker1
    participant Worker2
    participant Results
    
    Client->>Coordinator: Submit Task 1 (Echo)
    Client->>Coordinator: Submit Task 2 (Compute)
    Client->>Coordinator: Submit Task 3 (Error)
    
    Coordinator->>Worker1: Distribute Task 1
    Coordinator->>Worker2: Distribute Task 2
    Coordinator->>Worker1: Distribute Task 3
    
    Worker1->>Worker1: Process Echo (100ms)
    Worker2->>Worker2: Process Compute (200ms)
    Worker1->>Worker1: Process Error (50ms)
    
    Worker1->>Results: Task 1 Complete
    Worker2->>Results: Task 2 Complete
    Worker1->>Results: Task 3 Failed
    
    Results->>Coordinator: Report Results
    Coordinator->>Client: All Tasks Processed
    """
    
    # Generate diagrams
    diagrams = [
        (system_flow, "docs/images/system_flow.png", "System Flow Diagram"),
        (task_flow, "docs/images/task_flow.png", "Task Flow Sequence"),
    ]
    
    success_count = 0
    for mermaid_code, output_file, title in diagrams:
        if generate_diagram(mermaid_code, output_file, title):
            success_count += 1
    
    print(f"\n🎉 Generated {success_count}/{len(diagrams)} diagrams successfully!")
    print("📁 Diagrams saved in docs/images/")
    print("\n💡 You can now:")
    print("  - View diagrams in docs/images/")
    print("  - Open docs/diagram-viewer.html in your browser")
    print("  - Include images in presentations or documentation")

if __name__ == "__main__":
    main()
