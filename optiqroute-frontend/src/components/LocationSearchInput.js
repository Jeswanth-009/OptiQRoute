import React, { useState, useRef, useEffect } from 'react';
import useLocationSearch from '../hooks/useLocationSearch';
import './LocationSearchInput.css';

const LocationSearchInput = ({ 
  value, 
  onChange, 
  onLocationSelect, 
  placeholder, 
  icon,
  disabled = false 
}) => {
  const [inputValue, setInputValue] = useState(value || '');
  const [selectedLocation, setSelectedLocation] = useState(null);
  const inputRef = useRef(null);
  const dropdownRef = useRef(null);
  
  const {
    searchResults,
    isLoading,
    showDropdown,
    debouncedSearch,
    hideDropdown,
    clearResults
  } = useLocationSearch();

  useEffect(() => {
    setInputValue(value || '');
  }, [value]);

  useEffect(() => {
    const handleClickOutside = (event) => {
      if (
        dropdownRef.current && 
        !dropdownRef.current.contains(event.target) &&
        !inputRef.current.contains(event.target)
      ) {
        hideDropdown();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [hideDropdown]);

  const handleInputChange = (e) => {
    const newValue = e.target.value;
    setInputValue(newValue);
    onChange(newValue);
    
    if (newValue.length >= 3) {
      debouncedSearch(newValue);
    } else {
      clearResults();
    }
    
    // Clear selected location if input is manually changed
    if (selectedLocation && newValue !== selectedLocation.display_name) {
      setSelectedLocation(null);
    }
  };

  const handleLocationSelect = (location) => {
    setInputValue(location.display_name);
    setSelectedLocation(location);
    onChange(location.display_name);
    hideDropdown();
    
    // Call the parent callback with location data
    if (onLocationSelect) {
      onLocationSelect({
        address: location.display_name,
        coordinates: {
          lat: location.lat,
          lng: location.lon
        },
        addressComponents: location.address,
        placeId: location.id
      });
    }
  };

  const handleKeyDown = (e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (searchResults.length > 0) {
        handleLocationSelect(searchResults[0]);
      }
    } else if (e.key === 'Escape') {
      hideDropdown();
      inputRef.current?.blur();
    }
  };

  const handleFocus = () => {
    if (searchResults.length > 0) {
      // Show dropdown if there are existing results
      debouncedSearch(inputValue);
    }
  };

  return (
    <div className="location-search-container">
      <div className="input-with-icon">
        <input
          ref={inputRef}
          type="text"
          value={inputValue}
          onChange={handleInputChange}
          onKeyDown={handleKeyDown}
          onFocus={handleFocus}
          className={`form-input ${selectedLocation ? 'has-location' : ''}`}
          placeholder={placeholder}
          disabled={disabled}
          autoComplete="off"
        />
        <span className="input-icon">
          {isLoading ? '⏳' : icon}
        </span>
        {selectedLocation && (
          <span className="location-verified">✓</span>
        )}
      </div>
      
      {showDropdown && searchResults.length > 0 && (
        <div ref={dropdownRef} className="search-dropdown">
          {searchResults.map((result) => (
            <div
              key={result.id}
              className="search-result-item"
              onClick={() => handleLocationSelect(result)}
            >
              <div className="result-main">
                <span className="result-name">
                  {result.address.road || result.address.suburb || 'Location'}
                </span>
                {result.address.city && (
                  <span className="result-city">{result.address.city}</span>
                )}
              </div>
              <div className="result-address">
                {result.display_name}
              </div>
            </div>
          ))}
        </div>
      )}
      
      {showDropdown && searchResults.length === 0 && !isLoading && inputValue.length >= 3 && (
        <div ref={dropdownRef} className="search-dropdown">
          <div className="search-no-results">
            No locations found for "{inputValue}"
          </div>
        </div>
      )}
    </div>
  );
};

export default LocationSearchInput;
