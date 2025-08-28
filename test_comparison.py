import requests
import json

# Test the comparison endpoint with a simple CSV file upload simulation
def test_comparison_endpoint():
    print("🧪 Testing Comparison Endpoint with Address-based CSV...")
    
    # Create sample CSV data matching your format
    csv_data = '''start_location,end_location,end_address
"Visakhapatnam Airport, P6HG+6PR, Unnamed Road, Visakhapatnam Airport, Visakhapatnam, Andhra Pradesh 530009, India",VMRDA-Kailasagiri,"P8XR+HVC, Hill Top Rd, Kailasagiri, Visakhapatnam, Andhra Pradesh 530043, India"
"Visakhapatnam Airport, P6HG+6PR, Unnamed Road, Visakhapatnam Airport, Visakhapatnam, Andhra Pradesh 530009, India",Indira Gandhi Zoological Park,"near Dairy Farm, Yendada, Visakhapatnam, Andhra Pradesh 530040, India"
"Visakhapatnam Airport, P6HG+6PR, Unnamed Road, Visakhapatnam Airport, Visakhapatnam, Andhra Pradesh 530009, India",RK Beach,"RK Beach Rd, Visakhapatnam, Andhra Pradesh 530017, India"'''
    
    # Save to temporary file
    with open('temp_test.csv', 'w', encoding='utf-8') as f:
        f.write(csv_data)
    
    try:
        # Upload CSV file to comparison endpoint
        with open('temp_test.csv', 'rb') as f:
            files = {'file': ('test_routes.csv', f, 'text/csv')}
            
            response = requests.post(
                'http://localhost:5000/compare-route', 
                files=files, 
                timeout=120  # Longer timeout for geocoding
            )
        
        print(f"📊 Status: {response.status_code}")
        
        if response.status_code == 200:
            result = response.json()
            
            if result.get('success'):
                print("✅ Success! Comparison completed.")
                
                # Display results
                classical = result['results']['classical']
                quantum = result['results']['quantum']
                comparison = result['results'].get('comparison', {})
                
                print(f"\n📈 COMPARISON RESULTS:")
                print(f"Classical: {classical.get('distance_m', 0)/1000:.2f} km in {classical.get('solve_time_ms', 0):.1f}ms")
                print(f"Quantum:   {quantum.get('distance_m', 0)/1000:.2f} km in {quantum.get('solve_time_ms', 0):.1f}ms")
                
                if comparison:
                    print(f"\n🏆 Winner: {comparison.get('winner', 'unknown').upper()}")
                    print(f"📊 Improvement: {comparison.get('improvement_percent', 0):.2f}%")
                    print(f"⏱️ Time Ratio: {comparison.get('time_ratio', 0):.2f}x")
                
                # Show input data summary
                input_data = result.get('input_data', {})
                print(f"\n📍 Input Data:")
                print(f"Depot: {input_data.get('depot', 'N/A')}")
                print(f"Customers: {len(input_data.get('customers', []))} locations")
                
                # Show geocoding warnings if any
                warnings = result.get('geocoding_warnings', [])
                if warnings:
                    print(f"\n⚠️ Geocoding warnings: {len(warnings)} failed")
                    for warning in warnings[:3]:  # Show first 3
                        print(f"  - {warning}")
                
                print(f"\n🎉 CSV address format working perfectly!")
                
            else:
                print(f"❌ Request succeeded but analysis failed")
                print(f"Error details: {result}")
                
        else:
            print(f"❌ HTTP Error: {response.status_code}")
            print(f"Response: {response.text[:500]}")
            
    except Exception as e:
        print(f"❌ Request failed: {e}")
    
    finally:
        # Clean up temp file
        import os
        if os.path.exists('temp_test.csv'):
            os.remove('temp_test.csv')

if __name__ == "__main__":
    test_comparison_endpoint()
