import React, { useState } from 'react';
import { MapContainer, TileLayer, Polyline } from 'react-leaflet';
import 'leaflet/dist/leaflet.css';
import './App.css';

function App() {
  const [startLocation, setStartLocation] = useState('');
  const [fromLocation, setFromLocation] = useState('');
  const [toLocation, setToLocation] = useState('');
  const [deliveryPoints, setDeliveryPoints] = useState(['']);
  const [numVehicles, setNumVehicles] = useState(1);
  const [constraints, setConstraints] = useState('');
  const [routeCoords, setRouteCoords] = useState([]);
  const [mode, setMode] = useState('quantum');
  const [activeRouteTab, setActiveRouteTab] = useState('classical');
  const [results, setResults] = useState(null); // Start with no results

  const handleMapClick = (e) => {
    // Map interaction logic here
    console.log('Map clicked at:', e.latlng);
  };

  const optimizeRoute = async (type) => {
    try {
      // Validate based on the current mode and optimization type
      if (type === 'quantum') {
        // For quantum mode, we need start location and delivery points
        if (!startLocation.trim()) {
          alert('Please enter a start location.');
          return;
        }

        // Filter out empty delivery points
        const validDeliveryPoints = deliveryPoints.filter(point => point.trim());
        
        if (validDeliveryPoints.length === 0) {
          alert('Please enter at least one delivery point.');
          return;
        }

        // Calculate quantum optimization metrics
        const numDeliveries = validDeliveryPoints.length;
        const baseDistance = numDeliveries * 5; // Rough estimate: 5km per delivery
        const quantumImprovement = 0.85; // Quantum is 15% better
        
        const totalDistance = Math.round(baseDistance * quantumImprovement);
        const timeSaved = Math.round((baseDistance - totalDistance) * 0.1 * 10) / 10;
        
        setResults({
          totalDistance: `${totalDistance} km`,
          timeSaved: `${timeSaved} hours`,
          numDeliveries: numDeliveries,
          performanceImprovement: '15%',
          deliveryDistribution: '100%',
          historicalTrend: '25%',
          optimizationType: type
        });

      } else {
        // For classical mode, we need from and to locations
        if (!fromLocation.trim()) {
          alert('Please enter a from location.');
          return;
        }

        if (!toLocation.trim()) {
          alert('Please enter a to location.');
          return;
        }

        // Calculate classical optimization metrics
        const baseDistance = 25; // Fixed distance for point-to-point
        const totalDistance = Math.round(baseDistance);
        const estimatedTime = Math.round(baseDistance * 0.08 * 10) / 10;
        
        setResults({
          totalDistance: `${totalDistance} km`,
          timeSaved: `${estimatedTime} hours`,
          numDeliveries: 1,
          performanceImprovement: '8%',
          deliveryDistribution: '100%',
          historicalTrend: '18%',
          optimizationType: type
        });
      }

      // Show loading state (optional)
      console.log(`Optimizing route using ${type} algorithm...`);

      // Simulate some route coordinates for the map (mock data)
      const mockRoute = [
        [17.6868, 83.2185], // Visakhapatnam center
        [17.7068, 83.2285], // Point 1
        [17.7168, 83.2385], // Point 2
        [17.6968, 83.2485], // Point 3
        [17.6868, 83.2185]  // Back to start
      ];
      setRouteCoords(mockRoute);
      
    } catch (err) {
      console.error('Error optimizing route:', err);
      alert('An error occurred while optimizing the route. Please try again.');
    }
  };

  const resetAll = () => {
    setStartLocation('');
    setFromLocation('');
    setToLocation('');
    setDeliveryPoints(['']);
    setNumVehicles(1);
    setConstraints('');
    setRouteCoords([]);
    setResults(null); // Clear results when resetting
  };

  const addDeliveryPoint = () => {
    setDeliveryPoints([...deliveryPoints, '']);
  };

  const updateDeliveryPoint = (index, value) => {
    const newPoints = [...deliveryPoints];
    newPoints[index] = value;
    setDeliveryPoints(newPoints);
  };

  return (
    <div className="app-container">
      {/* Header */}
      <header className="header">
        <div className="header-left">
          <div className="logo"></div>
          <h1 className="app-title">OptiQRoute</h1>
        </div>
        <nav className="nav">
          <button className="nav-btn">About</button>
          <button className="nav-btn">Documentation</button>
          <button className="nav-btn">Contact</button>
          <button className="nav-btn">Team Credits</button>
        </nav>
      </header>

      {/* Main Content */}
      <div className="main-content">
        {/* Left Panel - Input Panel */}
        <div className="left-panel">
          <h2 className="panel-title">Input Panel</h2>
          
          {/* Mode Selection */}
          <div className="mode-tabs">
            <button
              onClick={() => setMode('quantum')}
              className={`mode-tab ${mode === 'quantum' ? 'active' : ''}`}
            >
              Quantum
            </button>
            <button
              onClick={() => setMode('classical')}
              className={`mode-tab ${mode === 'classical' ? 'active' : ''}`}
            >
              Classical
            </button>
          </div>

          {/* Quantum Mode Form */}
          {mode === 'quantum' && (
            <div className="form-content">
              {/* Start Location */}
              <div className="form-group">
                <label className="form-label">Start Location</label>
                <input
                  type="text"
                  value={startLocation}
                  onChange={(e) => setStartLocation(e.target.value)}
                  className="form-input"
                  placeholder="Enter start location"
                />
              </div>

              {/* Delivery Points */}
              <div className="form-group">
                <label className="form-label">Enter delivery points (one per line)</label>
                {deliveryPoints.map((point, index) => (
                  <input
                    key={index}
                    type="text"
                    value={point}
                    onChange={(e) => updateDeliveryPoint(index, e.target.value)}
                    className="form-input delivery-point"
                    placeholder={`Delivery point ${index + 1}`}
                  />
                ))}
                <button onClick={addDeliveryPoint} className="add-point-btn">
                  + Add another delivery point
                </button>
              </div>

              {/* Number of Vehicles */}
              <div className="form-group">
                <label className="form-label">Number of Vehicles</label>
                <input
                  type="number"
                  value={numVehicles}
                  onChange={(e) => setNumVehicles(e.target.value)}
                  className="form-input"
                  min="1"
                />
              </div>

              {/* Constraints */}
              <div className="form-group">
                <label className="form-label">Constraints (optional)</label>
                <textarea
                  value={constraints}
                  onChange={(e) => setConstraints(e.target.value)}
                  className="form-input form-textarea"
                  placeholder="Enter any constraints..."
                />
              </div>

              {/* Optimize Button for Quantum */}
              <div className="button-group">
                <button
                  onClick={() => optimizeRoute('quantum')}
                  className="btn btn-primary"
                >
                  Optimize (Quantum)
                </button>
                <button onClick={resetAll} className="btn btn-outline">
                  Reset
                </button>
              </div>
            </div>
          )}

          {/* Classical Mode Form */}
          {mode === 'classical' && (
            <div className="form-content">
              {/* From Location */}
              <div className="form-group">
                <label className="form-label">From Location</label>
                <input
                  type="text"
                  value={fromLocation}
                  onChange={(e) => setFromLocation(e.target.value)}
                  className="form-input"
                  placeholder="Enter from location"
                />
              </div>

              {/* To Location */}
              <div className="form-group">
                <label className="form-label">To Location</label>
                <input
                  type="text"
                  value={toLocation}
                  onChange={(e) => setToLocation(e.target.value)}
                  className="form-input"
                  placeholder="Enter to location"
                />
              </div>

              {/* Number of Vehicles */}
              <div className="form-group">
                <label className="form-label">Number of Vehicles</label>
                <input
                  type="number"
                  value={numVehicles}
                  onChange={(e) => setNumVehicles(e.target.value)}
                  className="form-input"
                  min="1"
                />
              </div>

              {/* Constraints */}
              <div className="form-group">
                <label className="form-label">Constraints (optional)</label>
                <textarea
                  value={constraints}
                  onChange={(e) => setConstraints(e.target.value)}
                  className="form-input form-textarea"
                  placeholder="Enter any constraints..."
                />
              </div>

              {/* Optimize Button for Classical */}
              <div className="button-group">
                <button
                  onClick={() => optimizeRoute('classical')}
                  className="btn btn-secondary"
                >
                  Optimize (Classical)
                </button>
                <button onClick={resetAll} className="btn btn-outline">
                  Reset
                </button>
              </div>
            </div>
          )}

          {/* Map Interaction Info */}
          <div className="map-interaction-info">
            <h3 className="info-title">Map Interaction</h3>
            <p className="info-text">
              Use the map below to visually select locations by dropping markers. This method is an alternative to typing coordinates and can be used for both start and delivery points.
            </p>
          </div>
        </div>

        {/* Right Content */}
        <div className="right-content">
          {/* Route Visualization */}
          <div className="route-tabs">
            <h2 className="panel-title" style={{ marginBottom: 0, padding: '1rem 0' }}>Route Visualization</h2>
            <div className="route-tab-list">
              <button
                onClick={() => setActiveRouteTab('classical')}
                className={`route-tab ${activeRouteTab === 'classical' ? 'active' : ''}`}
              >
                Classical Routes
              </button>
              <button
                onClick={() => setActiveRouteTab('quantum')}
                className={`route-tab ${activeRouteTab === 'quantum' ? 'active' : ''}`}
              >
                Quantum Optimized Routes
              </button>
            </div>
          </div>

          {/* Map Container */}
          <div className="map-container">
            {/* Search Box Overlay */}
            <div className="map-search">
              <div className="search-box">
                <svg className="search-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
                <input
                  type="text"
                  placeholder="Search for a location"
                  className="search-input"
                />
              </div>
            </div>

            <MapContainer
              center={[17.6868, 83.2185]} // Visakhapatnam, Andhra Pradesh, India coordinates
              zoom={13}
              className="leaflet-container"
              onClick={handleMapClick}
            >
              <TileLayer
                attribution='© OpenStreetMap contributors'
                url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
              />
              {routeCoords.length > 0 && (
                <Polyline
                  positions={routeCoords}
                  color={activeRouteTab === 'classical' ? '#3b82f6' : '#8b5cf6'}
                  weight={5}
                />
              )}
            </MapContainer>
          </div>

          {/* Results Dashboard */}
          <div className="results-dashboard">
            <h3 className="dashboard-title">
              Results Dashboard
              {results && (
                <span style={{ 
                  fontSize: '0.75rem', 
                  marginLeft: '0.5rem', 
                  padding: '0.25rem 0.5rem', 
                  backgroundColor: results.optimizationType === 'quantum' ? '#8b5cf6' : '#3b82f6',
                  color: 'white',
                  borderRadius: '0.25rem',
                  textTransform: 'capitalize'
                }}>
                  {results.optimizationType}
                </span>
              )}
            </h3>
            
            {results ? (
              <>
                {/* Main Metrics */}
                <div className="metrics-grid">
                  <div className="metric">
                    <div className="metric-value">{results.totalDistance}</div>
                    <div className="metric-label">Total Route Distance</div>
                  </div>
                  <div className="metric">
                    <div className="metric-value">{results.timeSaved}</div>
                    <div className="metric-label">Time Saved</div>
                  </div>
                  <div className="metric">
                    <div className="metric-value">{results.numDeliveries}</div>
                    <div className="metric-label">Number of Deliveries</div>
                  </div>
                </div>

                {/* Analysis Sections */}
                <div className="analysis-grid">
                  <div className="analysis-section">
                    <h4 className="analysis-title">Performance Comparison</h4>
                    <div className="analysis-value performance-value">{results.performanceImprovement}</div>
                    <div className="analysis-change">Last Month +5%</div>
                  </div>

                  <div className="analysis-section">
                    <h4 className="analysis-title">Delivery Distribution</h4>
                    <div className="analysis-value delivery-value">{results.deliveryDistribution}</div>
                    <div className="analysis-change">Current +2%</div>
                    
                    <div className="vehicle-distribution">
                      <div className="vehicle-item">
                        <span className="vehicle-name">Vehicle A</span>
                        <div className="progress-bar">
                          <div className="progress-fill" style={{ width: '80%' }}></div>
                        </div>
                      </div>
                      <div className="vehicle-item">
                        <span className="vehicle-name">Vehicle B</span>
                        <div className="progress-bar">
                          <div className="progress-fill" style={{ width: '60%' }}></div>
                        </div>
                      </div>
                      <div className="vehicle-item">
                        <span className="vehicle-name">Vehicle C</span>
                        <div className="progress-bar">
                          <div className="progress-fill" style={{ width: '50%' }}></div>
                        </div>
                      </div>
                    </div>
                  </div>

                  <div className="analysis-section">
                    <div className="historical-header">
                      <h4 className="analysis-title">Historical Trends</h4>
                      <div className="historical-tabs">
                        <button className="historical-tab classical">Classical</button>
                        <button className="historical-tab quantum">Quantum</button>
                      </div>
                    </div>
                    <div className="analysis-value">{results.historicalTrend}</div>
                    <div className="analysis-change">Last 6 Months +10%</div>
                    
                    <div className="chart-container">
                      <svg className="chart-svg" viewBox="0 0 200 40">
                        <polyline
                          fill="none"
                          stroke="#3B82F6"
                          strokeWidth="2"
                          points="10,30 25,28 40,25 55,22 70,18 85,20 100,16 115,14 130,12 145,10 160,8 175,6 190,4"
                        />
                      </svg>
                    </div>
                  </div>
                </div>
              </>
            ) : (
              <div className="no-results">
                <div className="no-results-icon">
                  <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M9 11a3 3 0 016 0v1a3 3 0 01-6 0v-1z"/>
                    <path d="M12 1v3"/>
                    <path d="M21 9h-3"/>
                    <path d="M3 9h3"/>
                    <path d="M12 20v3"/>
                    <path d="M5.64 5.64l2.12 2.12"/>
                    <path d="M16.24 16.24l2.12 2.12"/>
                    <path d="M16.24 5.64l-2.12 2.12"/>
                    <path d="M5.64 16.24l2.12-2.12"/>
                  </svg>
                </div>
                <h4 className="no-results-title">No Routes Generated Yet</h4>
                <p className="no-results-text">
                  Enter your start location and delivery points, then click "Optimize" to generate routes and see detailed analytics.
                </p>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Footer */}
      <footer className="footer">
        <div className="footer-copyright">
          © 2025 OptiQRoute. All rights reserved.
        </div>
      </footer>
    </div>
  );
}

export default App;
