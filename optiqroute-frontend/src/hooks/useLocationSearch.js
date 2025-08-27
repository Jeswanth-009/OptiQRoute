import { useState, useCallback, useRef } from 'react';
import axios from 'axios';

const useLocationSearch = () => {
  const [searchResults, setSearchResults] = useState([]);
  const [isLoading, setIsLoading] = useState(false);
  const [showDropdown, setShowDropdown] = useState(false);
  const debounceRef = useRef(null);

  const searchLocations = useCallback(async (query) => {
    if (!query || query.length < 3) {
      setSearchResults([]);
      setShowDropdown(false);
      return;
    }

    setIsLoading(true);
    
    try {
      // Using Nominatim (OpenStreetMap) geocoding service
      const response = await axios.get('https://nominatim.openstreetmap.org/search', {
        params: {
          q: query,
          format: 'json',
          limit: 5,
          countrycodes: 'in', // Limit to India, you can remove this for worldwide search
          addressdetails: 1,
          extratags: 1
        }
      });

      const results = response.data.map(item => ({
        id: item.place_id,
        display_name: item.display_name,
        lat: parseFloat(item.lat),
        lon: parseFloat(item.lon),
        address: {
          house_number: item.address?.house_number || '',
          road: item.address?.road || '',
          suburb: item.address?.suburb || '',
          city: item.address?.city || item.address?.town || item.address?.village || '',
          state: item.address?.state || '',
          postcode: item.address?.postcode || '',
          country: item.address?.country || ''
        },
        type: item.type,
        importance: item.importance
      }));

      setSearchResults(results);
      setShowDropdown(true);
    } catch (error) {
      console.error('Error searching locations:', error);
      setSearchResults([]);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const debouncedSearch = useCallback((query) => {
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }
    
    debounceRef.current = setTimeout(() => {
      searchLocations(query);
    }, 300); // 300ms debounce
  }, [searchLocations]);

  const hideDropdown = useCallback(() => {
    setShowDropdown(false);
  }, []);

  const clearResults = useCallback(() => {
    setSearchResults([]);
    setShowDropdown(false);
  }, []);

  return {
    searchResults,
    isLoading,
    showDropdown,
    debouncedSearch,
    hideDropdown,
    clearResults
  };
};

export default useLocationSearch;
