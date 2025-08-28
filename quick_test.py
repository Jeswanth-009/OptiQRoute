import requests
import json

# Quick test
response = requests.post(
    "http://127.0.0.1:5000/quantum-route",
    json={
        "start": [17.6868, 83.2185],
        "deliveries": [[17.7000, 83.2300]],
        "num_vehicles": 1,
        "vehicle_capacity": 100
    }
)

print(f"Status: {response.status_code}")
if response.status_code == 200:
    result = response.json()
    print(f"✅ Quantum working: {result.get('algorithm')} in {result.get('solve_time_ms')}ms")
    print(f"Distance: {result.get('distance_m')} km")
else:
    print(f"❌ Error: {response.text}")
