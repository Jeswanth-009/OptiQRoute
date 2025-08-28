#!/usr/bin/env python3
"""
Enhanced test for the fixed quantum VRP endpoint
"""

import requests
import json
import time

def test_improved_quantum():
    url = "http://127.0.0.1:5000/quantum-route"
    
    # Test with 4 customers as mentioned by user
    test_data = {
        "start": [17.6868, 83.2185],  # Depot in Visakhapatnam
        "deliveries": [
            [17.7000, 83.2300],      # Customer 1  
            [17.6950, 83.2250],      # Customer 2
            [17.6800, 83.2100],      # Customer 3
            [17.7050, 83.2350]       # Customer 4
        ],
        "num_vehicles": 2,
        "vehicle_capacity": 100
    }
    
    print("🧪 Testing FIXED Quantum VRP Endpoint")
    print("=" * 60)
    print(f"📍 Testing with {len(test_data['deliveries'])} customers")
    print(f"🚛 Vehicles: {test_data['num_vehicles']}")
    print("=" * 60)
    
    try:
        print("⚛️  Sending quantum request...")
        start_time = time.time()
        
        response = requests.post(
            url, 
            json=test_data,
            headers={'Content-Type': 'application/json'},
            timeout=120  # Longer timeout for quantum processing
        )
        
        end_time = time.time()
        total_time = (end_time - start_time) * 1000  # ms
        
        print(f"📡 Response received in {total_time:.1f}ms")
        print(f"🔢 Status code: {response.status_code}")
        
        if response.status_code == 200:
            result = response.json()
            print("\n✅ QUANTUM VRP SUCCESS!")
            print("=" * 60)
            
            # Key metrics that should now work
            distance_m = result.get('distance_m', 0)
            distance_km = result.get('total_distance_km', 0)
            solve_time = result.get('solve_time_ms', 0)
            optimization_time = result.get('optimization_time_s', 0)
            
            print(f"📏 Distance: {distance_m:.1f} meters ({distance_km:.3f} km)")
            print(f"⏱️  Total response time: {total_time:.1f}ms")
            print(f"⚛️  Quantum solve time: {solve_time:.1f}ms")
            print(f"🔄 Optimization time: {optimization_time:.2f}s")
            print(f"🤖 Algorithm: {result.get('algorithm', 'Unknown')}")
            print(f"🚛 Vehicles used: {result.get('num_vehicles_used', 0)}")
            print(f"👥 Customers served: {result.get('customers_served', 0)}")
            print(f"🎯 Quantum advantage: {result.get('quantum_advantage', 'N/A')}")
            
            # Check if we have route data
            routes = result.get('routes', [])
            print(f"\n🛣️  Routes generated: {len(routes)}")
            for i, route in enumerate(routes):
                route_distance = route.get('distance', 0)
                customers_in_route = route.get('customers_served', 0)
                print(f"  Route {i+1}: {customers_in_route} customers, {route_distance:.3f} km")
            
            # Check quantum details
            quantum_details = result.get('quantum_details', {})
            if quantum_details:
                print(f"\n⚛️  Quantum Details:")
                print(f"  🔢 Bitstring: {quantum_details.get('bitstring', 'N/A')}")
                print(f"  📊 QAOA layers: {quantum_details.get('qaoa_layers', 'N/A')}")
                print(f"  🔄 Optimization iterations: {quantum_details.get('optimization_iterations', 'N/A')}")
                
            # Verify fixes
            print(f"\n🔍 Fix Verification:")
            print(f"  ✅ Distance > 0: {distance_m > 0}")
            print(f"  ✅ Time > 0: {solve_time > 0}")
            print(f"  ✅ Routes exist: {len(routes) > 0}")
            print(f"  ✅ Slow enough: {total_time > 2000}ms (should be >2s for small problems)")
            
        else:
            print(f"❌ Request failed!")
            print(f"Response: {response.text}")
            
    except Exception as e:
        print(f"💥 Error: {e}")

if __name__ == "__main__":
    print("🚀 Testing IMPROVED Quantum VRP Implementation")
    print("🎯 Fixes: Distance conversion, Route handling, Longer quantum time")
    test_improved_quantum()
    print("\n" + "=" * 60)
    print("✨ Test completed!")
