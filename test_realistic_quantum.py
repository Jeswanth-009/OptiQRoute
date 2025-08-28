#!/usr/bin/env python3
"""
Test the FIXED quantum VRP to ensure it's slower and less optimal than classical
"""

import requests
import json
import time

def test_realistic_quantum():
    """Test quantum with realistic performance expectations"""
    url = "http://127.0.0.1:5000/quantum-route"
    
    # Same test data as classical for comparison
    test_data = {
        "start": [17.6868, 83.2185],  # Depot
        "deliveries": [
            [17.7000, 83.2300],      # Customer 1
            [17.6950, 83.2250],      # Customer 2
            [17.6800, 83.2100],      # Customer 3
            [17.7100, 83.2400]       # Customer 4
        ],
        "num_vehicles": 2,
        "vehicle_capacity": 100
    }
    
    print("🧪 Testing REALISTIC Quantum VRP Performance")
    print("🎯 Expected: Slower than classical, potentially less optimal")
    print("=" * 60)
    print(f"📍 Testing with {len(test_data['deliveries'])} customers")
    print(f"🚛 Vehicles: {test_data['num_vehicles']}")
    print("=" * 60)
    
    try:
        print("⚛️  Sending quantum request...")
        start_time = time.time()
        
        response = requests.post(url, json=test_data, timeout=120)  # Longer timeout
        
        end_time = time.time()
        response_time = (end_time - start_time) * 1000  # ms
        
        print(f"📡 Response received in {response_time:.1f}ms")
        print(f"🔢 Status code: {response.status_code}")
        
        if response.status_code == 200:
            result = response.json()
            
            print("\n✅ QUANTUM VRP RESULTS:")
            print("=" * 60)
            print(f"📏 Distance: {result.get('distance_m', 0):.1f} meters ({result.get('distance_m', 0)/1000:.3f} km)")
            print(f"⏱️  Total response time: {response_time:.1f}ms")
            print(f"⚛️  Quantum solve time: {result.get('solve_time_ms', 0):.1f}ms")
            print(f"🔄 Optimization time: {result.get('solve_time_ms', 0)/1000:.1f}s")
            print(f"🤖 Algorithm: {result.get('algorithm', 'Unknown')}")
            print(f"🚛 Vehicles used: {result.get('num_vehicles_used', 0)}")
            print(f"👥 Customers served: {len(test_data['deliveries'])}")
            print(f"🎯 Quantum advantage: {result.get('quantum_advantage', 'N/A')}")
            
            if 'routes' in result and result['routes']:
                print(f"\n🛣️  Routes generated: {len(result['routes'])}")
                for i, route in enumerate(result['routes']):
                    print(f"  Route {i+1}: {route.get('customers_served', 0)} customers, {route.get('distance', 0):.3f} km")
            
            if 'quantum_details' in result:
                details = result['quantum_details']
                print(f"\n⚛️  Quantum Details:")
                print(f"  🔢 Bitstring: {details.get('bitstring', 'N/A')}")
                print(f"  📊 QAOA layers: {details.get('qaoa_layers', 'N/A')}")
                print(f"  🔄 Optimization iterations: {details.get('optimization_iterations', 'N/A')}")
                
            # Check if quantum behaves realistically
            print(f"\n🔍 Realism Check:")
            solve_time_s = result.get('solve_time_ms', 0) / 1000
            print(f"  ✅ Distance > 0: {result.get('distance_m', 0) > 0}")
            print(f"  ✅ Time > 0: {solve_time_s > 0}")
            print(f"  ✅ Routes exist: {len(result.get('routes', [])) > 0}")
            print(f"  ✅ Slow enough: {solve_time_s > 5.0}s (should be >5s for small problems)")
            
            # Expected quantum characteristics for small problems
            expected_slow = solve_time_s > 5.0
            if expected_slow:
                print(f"  🎯 REALISTIC: Quantum is appropriately slow for small problems")
            else:
                print(f"  ⚠️  WARNING: Quantum should be slower for small problems")
                
            return result
        else:
            print(f"❌ Request failed!")
            print(f"Response: {response.text}")
            return None
            
    except requests.exceptions.Timeout:
        print("⏱️  Request timed out - quantum taking realistic time!")
        return None
    except Exception as e:
        print(f"💥 Error: {e}")
        return None

def test_classical_comparison():
    """Test classical for comparison"""
    url = "http://127.0.0.1:5000/classical-route"
    
    test_data = {
        "start": [17.6868, 83.2185],
        "deliveries": [
            [17.7000, 83.2300],
            [17.6950, 83.2250],
            [17.6800, 83.2100],
            [17.7100, 83.2400]
        ],
        "num_vehicles": 2,
        "vehicle_capacity": 100
    }
    
    print("\n🏁 Classical Comparison Test")
    print("=" * 60)
    
    try:
        start_time = time.time()
        response = requests.post(url, json=test_data, timeout=30)
        end_time = time.time()
        response_time = (end_time - start_time) * 1000
        
        print(f"📡 Classical response in {response_time:.1f}ms")
        
        if response.status_code == 200:
            result = response.json()
            print(f"✅ Classical distance: {result.get('distance_m', 0):.1f} meters")
            print(f"⚡ Classical solve time: {result.get('solve_time_ms', response_time):.1f}ms")
            print(f"🚀 Classical optimization: {result.get('solve_time_ms', response_time)/1000:.1f}s")
            return result
        else:
            print(f"❌ Classical failed: {response.text}")
            return None
            
    except Exception as e:
        print(f"💥 Classical error: {e}")
        return None

if __name__ == "__main__":
    print("🚀 FIXED Quantum VRP Realism Test")
    print("Testing improved quantum behavior with realistic performance")
    
    # Test quantum
    quantum_result = test_realistic_quantum()
    
    # Test classical
    classical_result = test_classical_comparison()
    
    # Compare results
    if quantum_result and classical_result:
        print("\n" + "=" * 60)
        print("📊 QUANTUM vs CLASSICAL COMPARISON")
        print("=" * 60)
        
        q_dist = quantum_result.get('distance_m', 0)
        c_dist = classical_result.get('distance_m', 0)
        q_time = quantum_result.get('solve_time_ms', 0)
        c_time = classical_result.get('solve_time_ms', 0)
        
        print(f"Distance Comparison:")
        print(f"  🏛️  Classical: {c_dist:.1f}m")
        print(f"  ⚛️  Quantum:   {q_dist:.1f}m")
        print(f"  📈 Difference: {((q_dist - c_dist) / c_dist * 100):+.1f}%")
        
        print(f"\nTime Comparison:")
        print(f"  🏛️  Classical: {c_time:.1f}ms")
        print(f"  ⚛️  Quantum:   {q_time:.1f}ms")
        if c_time > 0:
            print(f"  📈 Difference: {((q_time - c_time) / c_time * 100):+.1f}%")
        else:
            print(f"  📈 Quantum is {q_time/1000:.1f}s vs Classical <0.1s (much slower)")
        
        # Check if quantum behavior is realistic
        quantum_slower = q_time > max(c_time * 5, 1000)  # At least 5x slower or 1s minimum
        quantum_worse = q_dist > c_dist * 0.95  # Should be similar or slightly worse
        
        print(f"\n🎯 REALISM CHECK:")
        print(f"  ✅ Quantum slower: {quantum_slower} (quantum: {q_time:.1f}ms vs classical: {c_time:.1f}ms)")
        print(f"  ✅ Quantum solution quality: {'Worse (realistic)' if q_dist > c_dist else 'Better (unexpected)'}")
        
        if quantum_slower and q_dist > c_dist:
            print(f"  🎉 SUCCESS: Quantum shows realistic behavior for small problems!")
            print(f"     - Quantum is slower (good for small problems)")
            print(f"     - Quantum found worse solution (realistic for NISQ devices)")
        else:
            print(f"  ⚠️  Quantum behavior analysis:")
            if not quantum_slower:
                print(f"     - Should be much slower for small problems")
            if q_dist <= c_dist:
                print(f"     - Solution unexpectedly good (might need more quantum noise)")
    
    print("\n✨ Test completed!")
    print("For 4 customers, quantum should be slower but demonstrate quantum principles")
