import React, { useState } from 'react';
import { MapContainer, TileLayer, Polyline, Marker, Popup } from 'react-leaflet';
import L from 'leaflet';
import 'leaflet/dist/leaflet.css';
import './App.css';

// Fix default markers
delete L.Icon.Default.prototype._getIconUrl;
L.Icon.Default.mergeOptions({
  iconRetinaUrl: require('leaflet/dist/images/marker-icon-2x.png'),
  iconUrl: require('leaflet/dist/images/marker-icon.png'),
  shadowUrl: require('leaflet/dist/images/marker-shadow.png'),
});

function App() {
  const [startLocation, setStartLocation] = useState('');
  const [endLocation, setEndLocation] = useState('');
  const [waypoints, setWaypoints] = useState(['']);
  const [optimizationMode, setOptimizationMode] = useState('classical');
  const [graphRadius, setGraphRadius] = useState(5);
  const [routeCoords, setRouteCoords] = useState([]);
  const [markers, setMarkers] = useState([]);
  const [results, setResults] = useState(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);

  const handleMapClick = (e) => {
    console.log('Map clicked at:', e.latlng);
    // Add marker at clicked location
    const newMarker = {
      position: [e.latlng.lat, e.latlng.lng],
      id: Date.now(),
      type: 'waypoint'
    };
    setMarkers([...markers, newMarker]);
  };

  const optimizeRoute = async () => {
    if (!startLocation.trim()) {
      alert('Please enter a start location.');
      return;
    }

    if (!endLocation.trim()) {
      alert('Please enter an end location.');
      return;
    }

    setIsAnalyzing(true);

    try {
      // Simulate API call delay
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // Calculate metrics based on optimization mode
      const baseDistance = 25 + (waypoints.filter(w => w.trim()).length * 8);
      const improvement = optimizationMode === 'quantum' ? 0.85 : 0.92;
      const totalDistance = Math.round(baseDistance * improvement);
      const timeSaved = Math.round((baseDistance - totalDistance) * 0.08 * 10) / 10;
      const numWaypoints = waypoints.filter(w => w.trim()).length;
      
      setResults({
        distance: `${totalDistance} km`,
        time: `${Math.round(totalDistance * 0.08 * 10) / 10} hours`,
        optimizationTime: optimizationMode === 'quantum' ? '2.3s' : '5.7s',
        nodesProcessed: optimizationMode === 'quantum' ? Math.floor(Math.random() * 500 + 1200) : Math.floor(Math.random() * 300 + 800),
        optimizationType: optimizationMode
      });

      // Generate mock route coordinates
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
    } finally {
      setIsAnalyzing(false);
    }
  };

  const addWaypoint = () => {
    setWaypoints([...waypoints, '']);
  };

  const updateWaypoint = (index, value) => {
    const newWaypoints = [...waypoints];
    newWaypoints[index] = value;
    setWaypoints(newWaypoints);
  };

  const removeWaypoint = (index) => {
    if (waypoints.length > 1) {
      const newWaypoints = waypoints.filter((_, i) => i !== index);
      setWaypoints(newWaypoints);
    }
  };

  const exportResults = () => {
    if (!results) return;
    
    const data = {
      route: routeCoords,
      results: results,
      timestamp: new Date().toISOString()
    };
    
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `route-optimization-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const tweakParameters = () => {
    // Simple parameter adjustment
    setGraphRadius(graphRadius === 5 ? 8 : 5);
    alert(`Graph radius updated to ${graphRadius === 5 ? 8 : 5} km`);
  };

  const compareOptimizations = () => {
    if (!results) return;
    
    alert(`Current: ${optimizationMode} optimization\nDistance: ${results.distance}\nTime: ${results.optimizationTime}\n\nWould show comparison with other optimization modes.`);
  };

  const reRunAnalysis = () => {
    optimizeRoute();
  };

  return (
    <div className="app-container">
      {/* Header */}
      <header className="header">
        <div className="header-content">
          <div className="logo-section">
            <div className="quantum-logo">
              <svg viewBox="0 0 24 24" className="atom-icon">
                <circle cx="12" cy="12" r="2" fill="currentColor"/>
                <path d="M12 2c5.5 0 10 4.5 10 10s-4.5 10-10 10S2 17.5 2 12 6.5 2 12 2z" fill="none" stroke="currentColor" strokeWidth="1"/>
                <ellipse cx="12" cy="12" rx="8" ry="3" fill="none" stroke="currentColor" strokeWidth="1" transform="rotate(45 12 12)"/>
                <ellipse cx="12" cy="12" rx="8" ry="3" fill="none" stroke="currentColor" strokeWidth="1" transform="rotate(-45 12 12)"/>
              </svg>
            </div>
            <h1 className="app-title">Quantum Route Optimizer</h1>
          </div>
          <div className="header-actions">
            <button className="export-btn" onClick={exportResults} disabled={!results}>
              📤 Export
            </button>
            <button className="run-analysis-btn" onClick={optimizeRoute} disabled={isAnalyzing}>
              {isAnalyzing ? '⏳ Analyzing...' : '▶️ Run Analysis'}
            </button>
            <button className="help-btn">❓ Help</button>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <div className="main-content">
        {/* Left Panel - Input Configuration */}
        <div className="left-panel">
          <div className="panel-header">
            <h2 className="panel-title">Input Configuration</h2>
          </div>
          
          <div className="form-content">
            {/* Start Location */}
            <div className="form-group">
              <label className="form-label">Start Location</label>
              <div className="input-with-icon">
                <input
                  type="text"
                  value={startLocation}
                  onChange={(e) => setStartLocation(e.target.value)}
                  className="form-input"
                  placeholder="Click on map or enter address"
                />
                <span className="input-icon">📍</span>
              </div>
            </div>

            {/* End Location */}
            <div className="form-group">
              <label className="form-label">End Location</label>
              <div className="input-with-icon">
                <input
                  type="text"
                  value={endLocation}
                  onChange={(e) => setEndLocation(e.target.value)}
                  className="form-input"
                  placeholder="Click on map or enter address"
                />
                <span className="input-icon">🏁</span>
              </div>
            </div>

            {/* Waypoints */}
            <div className="form-group">
              <label className="form-label">
                Waypoints
                <button className="add-waypoint-btn" onClick={addWaypoint}>
                  + Add waypoints by clicking map
                </button>
              </label>
              {waypoints.map((waypoint, index) => (
                <div key={index} className="waypoint-input">
                  <input
                    type="text"
                    value={waypoint}
                    onChange={(e) => updateWaypoint(index, e.target.value)}
                    className="form-input"
                    placeholder={`Waypoint ${index + 1}`}
                  />
                  {waypoints.length > 1 && (
                    <button 
                      className="remove-waypoint-btn"
                      onClick={() => removeWaypoint(index)}
                    >
                      ×
                    </button>
                  )}
                </div>
              ))}
            </div>

            {/* Optimization Mode */}
            <div className="form-group">
              <label className="form-label">Optimization Mode</label>
              <div className="radio-group">
                <label className="radio-option">
                  <input
                    type="radio"
                    name="optimization"
                    value="classical"
                    checked={optimizationMode === 'classical'}
                    onChange={(e) => setOptimizationMode(e.target.value)}
                  />
                  <span className="radio-label">
                    <span className="radio-title">Classical</span>
                    <span className="radio-desc">Traditional graph algorithms</span>
                  </span>
                </label>
                <label className="radio-option">
                  <input
                    type="radio"
                    name="optimization"
                    value="quantum"
                    checked={optimizationMode === 'quantum'}
                    onChange={(e) => setOptimizationMode(e.target.value)}
                  />
                  <span className="radio-label">
                    <span className="radio-title">Quantum</span>
                    <span className="radio-desc">QUBO + Grover optimization</span>
                  </span>
                </label>
                <label className="radio-option">
                  <input
                    type="radio"
                    name="optimization"
                    value="compare"
                    checked={optimizationMode === 'compare'}
                    onChange={(e) => setOptimizationMode(e.target.value)}
                  />
                  <span className="radio-label">
                    <span className="radio-title">Compare Both</span>
                    <span className="radio-desc">Run quantum analysis</span>
                  </span>
                </label>
              </div>
            </div>

            {/* Parameters */}
            <div className="form-group">
              <label className="form-label">Parameters</label>
              <div className="parameter-group">
                <label className="parameter-label">
                  Optimization Goal
                  <select className="form-select">
                    <option>Shortest Distance</option>
                    <option>Fastest Time</option>
                    <option>Fuel Efficiency</option>
                  </select>
                </label>
                <label className="parameter-label">
                  Graph Radius (km)
                  <input
                    type="number"
                    value={graphRadius}
                    onChange={(e) => setGraphRadius(Number(e.target.value))}
                    className="form-input"
                    min="1"
                    max="20"
                  />
                </label>
              </div>
            </div>
          </div>
        </div>

        {/* Center - Interactive Map */}
        <div className="map-section">
          <div className="map-header">
            <h2 className="map-title">Interactive Map (OSM + Leaflet)</h2>
            <div className="map-controls">
              <button className="map-control-btn">+</button>
              <button className="map-control-btn">-</button>
              <button className="map-control-btn">⌂</button>
            </div>
          </div>
          
          <div className="map-container">
            <div className="map-overlay">
              <div className="map-status">
                <span className="status-text">Click to add waypoints</span>
                <span className="status-text">Right click to remove</span>
              </div>
            </div>

            <MapContainer
              center={[17.6868, 83.2185]}
              zoom={13}
              className="leaflet-container"
              onClick={handleMapClick}
            >
              <TileLayer
                attribution='© OpenStreetMap contributors'
                url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
              />
              
              {/* Route Polyline */}
              {routeCoords.length > 0 && (
                <Polyline
                  positions={routeCoords}
                  color={optimizationMode === 'quantum' ? '#8b5cf6' : '#3b82f6'}
                  weight={4}
                  opacity={0.8}
                />
              )}
              
              {/* Markers */}
              {markers.map((marker) => (
                <Marker key={marker.id} position={marker.position}>
                  <Popup>
                    {marker.type === 'start' ? 'Start Location' : 
                     marker.type === 'end' ? 'End Location' : 'Waypoint'}
                  </Popup>
                </Marker>
              ))}
            </MapContainer>

            <div className="map-legend">
              <div className="legend-item">
                <div className="legend-color start-point"></div>
                <span>Start Point</span>
              </div>
              <div className="legend-item">
                <div className="legend-color end-point"></div>
                <span>End Point</span>
              </div>
              <div className="legend-item">
                <div className="legend-color waypoints"></div>
                <span>Waypoints</span>
              </div>
            </div>
          </div>
        </div>

        {/* Right Panel - Results Dashboard */}
        <div className="right-panel">
          <div className="panel-header">
            <h2 className="panel-title">Results Dashboard</h2>
            <div className="status-indicator">
              <span className={`status-dot ${results ? 'ready' : 'waiting'}`}></span>
              <span className="status-text">
                {isAnalyzing ? 'Analyzing...' : results ? 'Ready for Input' : 'Last run: Never'}
              </span>
            </div>
          </div>

          {results ? (
            <div className="results-content">
              {/* Route Comparison */}
              <div className="results-section">
                <h3 className="section-title">Route Comparison</h3>
                <div className="comparison-tabs">
                  <button className={`tab-btn ${optimizationMode === 'classical' ? 'active' : ''}`}>
                    Classical
                  </button>
                  <button className={`tab-btn ${optimizationMode === 'quantum' ? 'active' : ''}`}>
                    Quantum
                  </button>
                </div>
                <div className="route-visualization">
                  <p>Route visualization will appear here</p>
                </div>
              </div>

              {/* Performance Metrics */}
              <div className="results-section">
                <h3 className="section-title">Performance Metrics</h3>
                <div className="metrics-grid">
                  <div className="metric-card">
                    <div className="metric-value">{results.distance}</div>
                    <div className="metric-label">Distance</div>
                  </div>
                  <div className="metric-card">
                    <div className="metric-value">{results.time}</div>
                    <div className="metric-label">Time</div>
                  </div>
                  <div className="metric-card">
                    <div className="metric-value">{results.optimizationTime}</div>
                    <div className="metric-label">Optimization Time</div>
                  </div>
                  <div className="metric-card">
                    <div className="metric-value">{results.nodesProcessed}</div>
                    <div className="metric-label">Nodes Processed</div>
                  </div>
                </div>
              </div>

              {/* Actions */}
              <div className="results-section">
                <h3 className="section-title">Actions</h3>
                <div className="action-buttons">
                  <button className="action-btn primary" onClick={reRunAnalysis}>
                    🔄 Re-run Analysis
                  </button>
                  <button className="action-btn secondary" onClick={tweakParameters}>
                    🎛️ Tweak Parameters
                  </button>
                  <button className="action-btn secondary" onClick={exportResults}>
                    📤 Export Results
                  </button>
                  <button className="action-btn secondary" onClick={compareOptimizations}>
                    📊 Compare Modes
                  </button>
                </div>
              </div>
            </div>
          ) : (
            <div className="no-results">
              <div className="no-results-content">
                <div className="no-results-icon">⏱️</div>
                <h3 className="no-results-title">Ready for Input</h3>
                <p className="no-results-description">
                  Optimization results and performance analytics will appear here after running analysis.
                </p>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Footer */}
      <footer className="footer">
        <div className="footer-content">
          <div className="footer-left">
            <span>Powered by OSMnx, Qiskit & Classical Optimization</span>
          </div>
          <div className="footer-right">
            <span>Status: Ready • Backend: Connected</span>
          </div>
        </div>
      </footer>
    </div>
  );
}

export default App;