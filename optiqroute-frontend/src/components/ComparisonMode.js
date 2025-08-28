import React, { useState } from 'react';
import './ComparisonMode.css';

const ComparisonMode = ({ onBack }) => {
  const [csvFile, setCsvFile] = useState(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [results, setResults] = useState(null);
  const [error, setError] = useState('');

  const handleFileChange = (event) => {
    const file = event.target.files[0];
    if (file && file.type === 'text/csv') {
      setCsvFile(file);
      setError('');
    } else {
      setError('Please select a valid CSV file');
      setCsvFile(null);
    }
  };

  const handleCompareAnalysis = async () => {
    if (!csvFile) {
      setError('Please select a CSV file first');
      return;
    }

    setIsAnalyzing(true);
    setError('');

    try {
      const formData = new FormData();
      formData.append('file', csvFile);

      const response = await fetch('http://localhost:5000/compare-route', {
        method: 'POST',
        body: formData
      });

      if (!response.ok) {
        throw new Error(`Server error: ${response.status}`);
      }

      const result = await response.json();
      
      if (result.success) {
        setResults(result);
      } else {
        setError(result.error || 'Analysis failed');
      }

    } catch (error) {
      console.error('Error during comparison:', error);
      setError(error.message || 'Failed to analyze routes. Please check your CSV format.');
    } finally {
      setIsAnalyzing(false);
    }
  };

  const downloadSampleCSV = () => {
    const sampleData = `lat,lon,type,demand
17.6868,83.2185,depot,0
17.6970,83.2095,customer,10
17.6970,83.2275,customer,15
17.6770,83.2275,customer,12
17.6770,83.2095,customer,8`;

    const blob = new Blob([sampleData], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'sample_route_data.csv';
    a.click();
    URL.revokeObjectURL(url);
  };

  const clearResults = () => {
    setResults(null);
    setCsvFile(null);
    setError('');
  };

  const formatDistance = (meters) => {
    return `${(meters / 1000).toFixed(2)} km`;
  };

  const formatTime = (milliseconds) => {
    return `${(milliseconds / 1000).toFixed(2)}s`;
  };

  return (
    <div className="comparison-mode">
      <div className="comparison-header">
        <button className="back-btn" onClick={onBack}>
          ← Back to Main
        </button>
        <h1 className="comparison-title">Quantum vs Classical Route Comparison</h1>
      </div>

      <div className="comparison-content">
        {/* File Upload Section */}
        <div className="upload-section">
          <div className="upload-card">
            <h2>Upload Route Data</h2>
            
            <div className="file-upload-area">
              <input
                type="file"
                id="csvFile"
                accept=".csv"
                onChange={handleFileChange}
                className="file-input"
              />
              <label htmlFor="csvFile" className="file-upload-label">
                <div className="upload-icon">📁</div>
                <div className="upload-text">
                  {csvFile ? csvFile.name : 'Choose CSV file'}
                </div>
                <div className="upload-hint">
                  Click to browse or drag & drop your CSV file
                </div>
              </label>
            </div>

            <div className="csv-requirements">
              <h3>CSV Format Requirements:</h3>
              <ul>
                <li><strong>Columns:</strong> lat, lon, type, demand (optional)</li>
                <li><strong>Types:</strong> "depot" for warehouse, "customer" for delivery points</li>
                <li><strong>Example:</strong> 17.6868,83.2185,depot,0</li>
                <li><strong>Minimum:</strong> 1 depot + 2 customers</li>
              </ul>
              <button className="sample-btn" onClick={downloadSampleCSV}>
                📥 Download Sample CSV
              </button>
            </div>

            {error && (
              <div className="error-message">
                ⚠️ {error}
              </div>
            )}

            <button 
              className={`analyze-btn ${csvFile ? 'enabled' : 'disabled'}`}
              onClick={handleCompareAnalysis}
              disabled={!csvFile || isAnalyzing}
            >
              {isAnalyzing ? '⏳ Analyzing...' : '🔬 Run Comparison Analysis'}
            </button>
          </div>
        </div>

        {/* Results Section */}
        {results && (
          <div className="results-section">
            <div className="results-header">
              <h2>Analysis Results</h2>
              <button className="clear-btn" onClick={clearResults}>
                🗑️ Clear Results
              </button>
            </div>

            <div className="comparison-grid">
              {/* Classical Results */}
              <div className="algorithm-card classical">
                <div className="card-header">
                  <h3>🔧 Classical Algorithm</h3>
                  <span className="algorithm-name">{results.results.classical.algorithm}</span>
                </div>
                <div className="metrics">
                  <div className="metric">
                    <span className="metric-label">Distance:</span>
                    <span className="metric-value">{formatDistance(results.results.classical.distance_m)}</span>
                  </div>
                  <div className="metric">
                    <span className="metric-label">Solve Time:</span>
                    <span className="metric-value">{formatTime(results.results.classical.solve_time_ms)}</span>
                  </div>
                  <div className="metric">
                    <span className="metric-label">Vehicles Used:</span>
                    <span className="metric-value">{results.results.classical.num_vehicles_used}</span>
                  </div>
                  <div className="metric">
                    <span className="metric-label">Solver:</span>
                    <span className="metric-value">{results.results.classical.solver}</span>
                  </div>
                </div>
              </div>

              {/* Quantum Results */}
              <div className="algorithm-card quantum">
                <div className="card-header">
                  <h3>⚛️ Quantum Algorithm</h3>
                  <span className="algorithm-name">{results.results.quantum.algorithm}</span>
                </div>
                <div className="metrics">
                  <div className="metric">
                    <span className="metric-label">Distance:</span>
                    <span className="metric-value">{formatDistance(results.results.quantum.distance_m)}</span>
                  </div>
                  <div className="metric">
                    <span className="metric-label">Solve Time:</span>
                    <span className="metric-value">{formatTime(results.results.quantum.solve_time_ms)}</span>
                  </div>
                  <div className="metric">
                    <span className="metric-label">Vehicles Used:</span>
                    <span className="metric-value">{results.results.quantum.num_vehicles_used}</span>
                  </div>
                  <div className="metric">
                    <span className="metric-label">Advantage:</span>
                    <span className="metric-value">{results.results.quantum.quantum_advantage || 'N/A'}</span>
                  </div>
                </div>
              </div>
            </div>

            {/* Comparison Summary */}
            {results.results.comparison && (
              <div className="comparison-summary">
                <h3>📊 Comparison Summary</h3>
                <div className="summary-grid">
                  <div className="summary-item">
                    <span className="summary-label">Distance Improvement:</span>
                    <span className={`summary-value ${results.results.comparison.improvement_percent > 0 ? 'positive' : 'negative'}`}>
                      {results.results.comparison.improvement_percent > 0 ? '+' : ''}{results.results.comparison.improvement_percent.toFixed(2)}%
                    </span>
                  </div>
                  <div className="summary-item">
                    <span className="summary-label">Time Ratio (Q/C):</span>
                    <span className="summary-value">
                      {results.results.comparison.time_ratio.toFixed(2)}x
                    </span>
                  </div>
                  <div className="summary-item">
                    <span className="summary-label">Winner:</span>
                    <span className={`summary-value ${results.results.comparison.winner}`}>
                      {results.results.comparison.winner === 'quantum' ? '⚛️ Quantum' : '🔧 Classical'}
                    </span>
                  </div>
                </div>
              </div>
            )}

            {/* Input Data Summary */}
            <div className="input-summary">
              <h3>📍 Input Data</h3>
              <div className="input-data">
                <p><strong>Depot:</strong> [{results.input_data.depot.join(', ')}]</p>
                <p><strong>Customers:</strong> {results.input_data.customers.length} locations</p>
                <p><strong>Vehicles:</strong> {results.input_data.num_vehicles}</p>
                <p><strong>Capacity:</strong> {results.input_data.vehicle_capacity} units</p>
              </div>
            </div>

            {/* Export Options */}
            <div className="export-section">
              <h3>📤 Export Results</h3>
              <div className="export-buttons">
                <button 
                  className="export-btn"
                  onClick={() => {
                    const blob = new Blob([JSON.stringify(results, null, 2)], { type: 'application/json' });
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = `route_comparison_${Date.now()}.json`;
                    a.click();
                    URL.revokeObjectURL(url);
                  }}
                >
                  📄 Export JSON
                </button>
                <button 
                  className="export-btn"
                  onClick={() => {
                    const csvData = [
                      ['Algorithm', 'Distance (km)', 'Time (s)', 'Vehicles', 'Solver'],
                      ['Classical', results.results.comparison.classical_distance_km, (results.results.classical.solve_time_ms/1000).toFixed(3), results.results.classical.num_vehicles_used, results.results.classical.solver],
                      ['Quantum', results.results.comparison.quantum_distance_km, (results.results.quantum.solve_time_ms/1000).toFixed(3), results.results.quantum.num_vehicles_used, results.results.quantum.solver]
                    ].map(row => row.join(',')).join('\n');

                    const blob = new Blob([csvData], { type: 'text/csv' });
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = `route_comparison_${Date.now()}.csv`;
                    a.click();
                    URL.revokeObjectURL(url);
                  }}
                >
                  📊 Export CSV
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default ComparisonMode;
