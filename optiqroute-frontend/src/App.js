import React, { useState, useEffect } from 'react';
import { MapContainer, TileLayer, Polyline, Marker, Popup } from 'react-leaflet';
import L from 'leaflet';
import 'leaflet/dist/leaflet.css';
import './App.css';
import LocationSearchInput from './components/LocationSearchInput';
import locationStorage from './services/locationStorage';
import MapClickHandler from './components/MapClickHandler';

// Fix default markers
delete L.Icon.Default.prototype._getIconUrl;
L.Icon.Default.mergeOptions({
  iconRetinaUrl: require('leaflet/dist/images/marker-icon-2x.png'),
  iconUrl: require('leaflet/dist/images/marker-icon.png'),
  shadowUrl: require('leaflet/dist/images/marker-shadow.png'),
});

// Create custom colored marker icons
const createCustomIcon = (color) => {
  return L.divIcon({
    className: 'custom-marker',
    html: `<div style="
      background-color: ${color};
      width: 25px;
      height: 25px;
      border-radius: 50% 50% 50% 0;
      border: 3px solid white;
      transform: rotate(-45deg);
      box-shadow: 0 2px 6px rgba(0,0,0,0.3);
    "><div style="
      width: 15px;
      height: 15px;
      background-color: white;
      border-radius: 50%;
      position: absolute;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%) rotate(45deg);
    "></div></div>`,
    iconSize: [25, 25],
    iconAnchor: [12, 25],
    popupAnchor: [0, -25]
  });
};

// Define marker icons with correct colors
const markerIcons = {
  depot: createCustomIcon('#10b981'),      // Green for depot/warehouse
  customer: createCustomIcon('#3b82f6'),   // Blue for customer locations
  temporary: createCustomIcon('#9ca3af'),  // Gray for temporary markers
  processing: createCustomIcon('#f59e0b'), // Orange for processing markers
  route: createCustomIcon('#8b5cf6')       // Purple for route points
};

function App() {
  const [startLocation, setStartLocation] = useState('');
  const [customerLocations, setCustomerLocations] = useState(['', '']);
  const [optimizationMode, setOptimizationMode] = useState('classical');
  const [graphRadius, setGraphRadius] = useState(5);
  const [routeCoords, setRouteCoords] = useState([]);
  const [markers, setMarkers] = useState([]);
  const [results, setResults] = useState(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [waitingForMapClick, setWaitingForMapClick] = useState(null); // null, 'depot', or customer index
  const [successMessage, setSuccessMessage] = useState('');

  // Load stored locations on component mount
  useEffect(() => {
    // Clear any existing markers first
    setMarkers([]);
    
    // Clear location storage on page load to start fresh
    locationStorage.clear();
    
    // Subscribe to location changes
    const unsubscribe = locationStorage.addListener((locations) => {
      updateMapMarkers(locations);
    });

    return unsubscribe;
  }, []);

  // Handle ESC key to cancel map click mode
  useEffect(() => {
    const handleKeyDown = (event) => {
      if (event.key === 'Escape' && waitingForMapClick !== null) {
        cancelMapClick();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [waitingForMapClick]);

  // Update map markers based on stored locations
  const updateMapMarkers = (locations) => {
    const newMarkers = locationStorage.getMapMarkers();
    setMarkers(newMarkers);
  };

  const handleMapClick = async (e) => {
    console.log('Map clicked at:', e.latlng, 'Waiting for map click:', waitingForMapClick);
    
    // Only process clicks when in map-click mode
    if (waitingForMapClick !== null) {
      console.log('Processing map click for location selection');
      // Handle map click for location selection
      const lat = e.latlng.lat;
      const lng = e.latlng.lng;
      
      // Add temporary marker immediately for visual feedback
      const tempMarker = {
        position: [lat, lng],
        id: `temp_${Date.now()}`,
        type: 'processing',
        address: 'Getting address...'
      };
      setMarkers([...markers, tempMarker]);
      
      try {
        console.log('Attempting reverse geocoding...');
        // Try to get address from reverse geocoding
        const response = await fetch(
          `https://nominatim.openstreetmap.org/reverse?format=json&lat=${lat}&lon=${lng}&zoom=18&addressdetails=1`
        );
        const data = await response.json();
        console.log('Reverse geocoding response:', data);
        
        const address = data.display_name || `Lat: ${lat.toFixed(6)}, Lng: ${lng.toFixed(6)}`;
        
        const locationData = {
          address: address,
          coordinates: { lat, lng },
          addressComponents: data.address || {},
          placeId: `map_click_${Date.now()}`
        };

        console.log('Location data created:', locationData);

        if (waitingForMapClick === 'depot') {
          console.log('Setting depot location');
          handleDepotLocationSelect(locationData);
          setSuccessMessage('✅ Depot location set successfully!');
        } else if (typeof waitingForMapClick === 'number') {
          console.log('Setting customer location for index:', waitingForMapClick);
          handleCustomerLocationSelect(waitingForMapClick, locationData);
          setSuccessMessage(`✅ Customer ${waitingForMapClick + 1} location set successfully!`);
        }
        
      } catch (error) {
        console.warn('Reverse geocoding failed, using coordinates:', error);
        
        // Fallback to coordinates if reverse geocoding fails
        const address = `Lat: ${lat.toFixed(6)}, Lng: ${lng.toFixed(6)}`;
        const locationData = {
          address: address,
          coordinates: { lat, lng },
          addressComponents: {},
          placeId: `map_click_${Date.now()}`
        };

        console.log('Using fallback location data:', locationData);

        if (waitingForMapClick === 'depot') {
          handleDepotLocationSelect(locationData);
          setSuccessMessage('✅ Depot location set successfully!');
        } else if (typeof waitingForMapClick === 'number') {
          handleCustomerLocationSelect(waitingForMapClick, locationData);
          setSuccessMessage(`✅ Customer ${waitingForMapClick + 1} location set successfully!`);
        }
      }
      
      // Remove temporary marker
      setMarkers(prevMarkers => prevMarkers.filter(m => m.id !== tempMarker.id));
      
      // Reset waiting state
      console.log('Resetting waiting state');
      setWaitingForMapClick(null);
      
      // Clear success message after 3 seconds
      setTimeout(() => setSuccessMessage(''), 3000);
    } else {
      // Do nothing when not in map-click mode - no temporary markers
      console.log('Map clicked but not in map-click mode, ignoring click');
    }
  };

  // Handle depot location selection
  const handleDepotLocationSelect = (locationData) => {
    console.log('Setting depot location:', locationData);
    setStartLocation(locationData.address);
    locationStorage.setDepot(locationData);
  };

  // Handle customer location selection
  const handleCustomerLocationSelect = (index, locationData) => {
    console.log('Setting customer location:', index, locationData);
    const newCustomerLocations = [...customerLocations];
    newCustomerLocations[index] = locationData.address;
    setCustomerLocations(newCustomerLocations);
    locationStorage.setCustomer(index, locationData);
  };

  // Enable map-click mode for depot
  const enableDepotMapClick = () => {
    console.log('Enabling depot map click mode');
    setWaitingForMapClick('depot');
  };

  // Enable map-click mode for customer location
  const enableCustomerMapClick = (index) => {
    console.log('Enabling customer map click mode for index:', index);
    setWaitingForMapClick(index);
  };

  // Cancel map-click mode
  const cancelMapClick = () => {
    console.log('Canceling map click mode');
    setWaitingForMapClick(null);
  };

  // Clear all data and start fresh
  const clearAllData = () => {
    setStartLocation('');
    setCustomerLocations(['', '']);
    setMarkers([]);
    setRouteCoords([]);
    setResults(null);
    setWaitingForMapClick(null);
    setSuccessMessage('');
    locationStorage.clear();
  };

  const optimizeRoute = async () => {
    try {
      const routeData = locationStorage.getRouteData();
      console.log('Route data for optimization:', routeData);
      
      setIsAnalyzing(true);

      // Simulate API call delay
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // Calculate metrics based on optimization mode and number of customers
      const numCustomers = routeData.customers.length;
      const baseDistance = 15 + (numCustomers * 6);
      const improvement = optimizationMode === 'quantum' ? 0.82 : 0.90;
      const totalDistance = Math.round(baseDistance * improvement);
      
      setResults({
        distance: `${totalDistance} km`,
        time: `${Math.round(totalDistance * 0.08 * 10) / 10} hours`,
        optimizationTime: optimizationMode === 'quantum' ? '1.8s' : '4.2s',
        nodesProcessed: optimizationMode === 'quantum' ? Math.floor(Math.random() * 500 + 1200) : Math.floor(Math.random() * 300 + 800),
        optimizationType: optimizationMode,
        customersServed: numCustomers,
        coordinateData: routeData // Store coordinate data for external use
      });

      // Generate mock route coordinates from actual stored locations
      const routePoints = [
        [routeData.depot.lat, routeData.depot.lng], // Start at depot
        ...routeData.customers.map(customer => [customer.lat, customer.lng]), // Visit customers
        [routeData.depot.lat, routeData.depot.lng]  // Return to depot
      ];
      setRouteCoords(routePoints);
      
    } catch (error) {
      console.error('Error optimizing route:', error);
      alert(error.message || 'An error occurred while optimizing the route. Please ensure you have a depot and at least 2 customer locations.');
    } finally {
      setIsAnalyzing(false);
    }
  };

  const addCustomerLocation = () => {
    setCustomerLocations([...customerLocations, '']);
  };

  const updateCustomerLocation = (index, value) => {
    const newLocations = [...customerLocations];
    newLocations[index] = value;
    setCustomerLocations(newLocations);
  };

  const removeCustomerLocation = (index) => {
    if (customerLocations.length > 2) {
      const newLocations = customerLocations.filter((_, i) => i !== index);
      setCustomerLocations(newLocations);
      locationStorage.removeCustomer(index);
    }
  };

  const exportResults = () => {
    if (!results) return;
    
    const data = {
      route: routeCoords,
      results: results,
      locationData: locationStorage.exportData(),
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

  const showCoordinateData = () => {
    try {
      const routeData = locationStorage.getRouteData();
      const dataString = JSON.stringify(routeData, null, 2);
      console.log('Coordinate Data:', routeData);
      
      // Create a modal or alert to show the data
      const newWindow = window.open('', '_blank');
      newWindow.document.write(`
        <html>
          <head><title>Route Coordinate Data</title></head>
          <body>
            <h2>Route Optimization Data</h2>
            <pre style="background: #f5f5f5; padding: 20px; border-radius: 5px; overflow: auto;">
${dataString}
            </pre>
            <button onclick="window.close()">Close</button>
          </body>
        </html>
      `);
    } catch (error) {
      alert(error.message);
    }
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
            <button className="clear-btn" onClick={clearAllData}>
              🗑️ Clear All
            </button>
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
            {/* Success Message */}
            {successMessage && (
              <div className="success-message">
                {successMessage}
              </div>
            )}
            {/* Depot Location */}
            <div className="form-group">
              <label className="form-label">Depot/Warehouse Location</label>
              <div className="location-input-group">
                <LocationSearchInput
                  value={startLocation}
                  onChange={setStartLocation}
                  onLocationSelect={handleDepotLocationSelect}
                  placeholder="Search for depot/warehouse location"
                  icon="🏢"
                  disabled={waitingForMapClick === 'depot'}
                />
                <button 
                  className={`map-click-btn ${waitingForMapClick === 'depot' ? 'active' : ''}`}
                  onClick={waitingForMapClick === 'depot' ? cancelMapClick : enableDepotMapClick}
                  title={waitingForMapClick === 'depot' ? 'Cancel map click' : 'Click to select on map'}
                >
                  {waitingForMapClick === 'depot' ? '❌' : '🗺️'}
                </button>
              </div>
              {waitingForMapClick === 'depot' && (
                <div className="map-click-hint">
                  Click anywhere on the map to set depot location
                </div>
              )}
            </div>

            {/* Customer Locations */}
            <div className="form-group">
              <label className="form-label">
                Customer Locations
                <button className="add-customer-btn" onClick={addCustomerLocation}>
                  + Add customer location
                </button>
              </label>
              {customerLocations.map((location, index) => (
                <div key={index} className="customer-input">
                  <div className="location-input-row">
                    <div className="location-input-group">
                      <LocationSearchInput
                        value={location}
                        onChange={(value) => updateCustomerLocation(index, value)}
                        onLocationSelect={(locationData) => handleCustomerLocationSelect(index, locationData)}
                        placeholder={`Search for customer ${index + 1} location`}
                        icon="🏪"
                        disabled={waitingForMapClick === index}
                      />
                      <button 
                        className={`map-click-btn ${waitingForMapClick === index ? 'active' : ''}`}
                        onClick={waitingForMapClick === index ? cancelMapClick : () => enableCustomerMapClick(index)}
                        title={waitingForMapClick === index ? 'Cancel map click' : 'Click to select on map'}
                      >
                        {waitingForMapClick === index ? '❌' : '🗺️'}
                      </button>
                    </div>
                    {customerLocations.length > 2 && (
                      <button 
                        className="remove-customer-btn"
                        onClick={() => removeCustomerLocation(index)}
                      >
                        ×
                      </button>
                    )}
                  </div>
                  {waitingForMapClick === index && (
                    <div className="map-click-hint">
                      Click anywhere on the map to set customer {index + 1} location
                    </div>
                  )}
                </div>
              ))}
              <div className="customer-help-text">
                Minimum 2 customer locations required for route optimization
                <br />
                💡 <strong>Tip:</strong> Click 🗺️ button then click map to pick locations visually
              </div>
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
              {/* Debug info */}
              {waitingForMapClick !== null && (
                <span className="debug-info">
                  Waiting: {waitingForMapClick === 'depot' ? 'Depot' : `Customer ${waitingForMapClick + 1}`}
                </span>
              )}
            </div>
          </div>
          
          <div className="map-container">
            <div className="map-overlay">
              <div className="map-status">
                {waitingForMapClick !== null ? (
                  <>
                    <span className="status-text status-active">
                      🎯 {waitingForMapClick === 'depot' 
                        ? 'Click to set depot location' 
                        : `Click to set customer ${waitingForMapClick + 1} location`}
                    </span>
                    <span className="status-text">
                      Press ESC or click ❌ to cancel
                    </span>
                  </>
                ) : (
                  <>
                    <span className="status-text">Search locations in form or click 🗺️ to select on map</span>
                    <span className="status-text">Map clicking is disabled unless 🗺️ button is active</span>
                  </>
                )}
              </div>
            </div>

            <MapContainer
              center={[17.6868, 83.2185]}
              zoom={13}
              className={`leaflet-container ${waitingForMapClick !== null ? 'map-click-mode' : ''}`}
            >
              <MapClickHandler onMapClick={handleMapClick} />
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
                <Marker 
                  key={marker.id || marker.index} 
                  position={marker.position}
                  icon={markerIcons[marker.type] || markerIcons.customer}
                >
                  <Popup>
                    {marker.popup || marker.address || 'Location'}
                  </Popup>
                </Marker>
              ))}
            </MapContainer>

            <div className="map-legend">
              <div className="legend-item">
                <div className="legend-color depot-point"></div>
                <span>Depot/Warehouse</span>
              </div>
              <div className="legend-item">
                <div className="legend-color customer-point"></div>
                <span>Customer Locations</span>
              </div>
              <div className="legend-item">
                <div className="legend-color route-line"></div>
                <span>Optimized Route</span>
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
                    <div className="metric-value">{results.customersServed}</div>
                    <div className="metric-label">Customers Served</div>
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
                  <button className="action-btn secondary" onClick={showCoordinateData}>
                    📍 View Coordinates
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