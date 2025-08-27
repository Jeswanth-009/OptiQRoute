// LocationStorage service to manage location data with coordinates
class LocationStorageService {
  constructor() {
    this.locations = {
      depot: null,
      customers: []
    };
    this.listeners = [];
  }

  // Add a listener for location changes
  addListener(callback) {
    this.listeners.push(callback);
    return () => {
      this.listeners = this.listeners.filter(listener => listener !== callback);
    };
  }

  // Notify all listeners of changes
  notifyListeners() {
    this.listeners.forEach(callback => callback(this.locations));
  }

  // Set depot location
  setDepot(locationData) {
    this.locations.depot = {
      id: 'depot',
      address: locationData.address,
      coordinates: locationData.coordinates,
      addressComponents: locationData.addressComponents,
      placeId: locationData.placeId,
      timestamp: new Date().toISOString()
    };
    this.notifyListeners();
    this.saveToLocalStorage();
  }

  // Add or update customer location
  setCustomer(index, locationData) {
    while (this.locations.customers.length <= index) {
      this.locations.customers.push(null);
    }
    
    this.locations.customers[index] = {
      id: `customer_${index}`,
      index: index,
      address: locationData.address,
      coordinates: locationData.coordinates,
      addressComponents: locationData.addressComponents,
      placeId: locationData.placeId,
      timestamp: new Date().toISOString()
    };
    this.notifyListeners();
    this.saveToLocalStorage();
  }

  // Remove customer location
  removeCustomer(index) {
    if (index < this.locations.customers.length) {
      this.locations.customers.splice(index, 1);
      // Update indices for remaining customers
      this.locations.customers.forEach((customer, i) => {
        if (customer) {
          customer.index = i;
          customer.id = `customer_${i}`;
        }
      });
      this.notifyListeners();
      this.saveToLocalStorage();
    }
  }

  // Add new customer location at the end
  addCustomer(locationData) {
    const newIndex = this.locations.customers.length;
    this.setCustomer(newIndex, locationData);
  }

  // Get all locations with coordinates
  getAllLocations() {
    return {
      depot: this.locations.depot,
      customers: this.locations.customers.filter(customer => customer !== null)
    };
  }

  // Get locations formatted for route optimization algorithms
  getRouteData() {
    const data = this.getAllLocations();
    
    if (!data.depot) {
      throw new Error('Depot location is required');
    }

    const validCustomers = data.customers.filter(customer => 
      customer && customer.coordinates && customer.coordinates.lat && customer.coordinates.lng
    );

    if (validCustomers.length < 2) {
      throw new Error('At least 2 customer locations are required');
    }

    return {
      depot: {
        id: data.depot.id,
        lat: data.depot.coordinates.lat,
        lng: data.depot.coordinates.lng,
        address: data.depot.address
      },
      customers: validCustomers.map(customer => ({
        id: customer.id,
        index: customer.index,
        lat: customer.coordinates.lat,
        lng: customer.coordinates.lng,
        address: customer.address
      })),
      totalLocations: validCustomers.length + 1,
      coordinateMatrix: this.generateCoordinateMatrix(data.depot, validCustomers)
    };
  }

  // Generate coordinate matrix for algorithms
  generateCoordinateMatrix(depot, customers) {
    const allLocations = [depot, ...customers];
    return allLocations.map(location => [
      location.coordinates.lat,
      location.coordinates.lng
    ]);
  }

  // Get locations for map display
  getMapMarkers() {
    const locations = this.getAllLocations();
    const markers = [];

    if (locations.depot) {
      markers.push({
        id: 'depot',
        position: [locations.depot.coordinates.lat, locations.depot.coordinates.lng],
        type: 'depot',
        address: locations.depot.address,
        popup: `Depot: ${locations.depot.address}`
      });
    }

    locations.customers.forEach((customer, index) => {
      if (customer && customer.coordinates) {
        markers.push({
          id: customer.id,
          position: [customer.coordinates.lat, customer.coordinates.lng],
          type: 'customer',
          index: index,
          address: customer.address,
          popup: `Customer ${index + 1}: ${customer.address}`
        });
      }
    });

    return markers;
  }

  // Export data for external use
  exportData() {
    const routeData = this.getRouteData();
    const exportData = {
      ...routeData,
      exportedAt: new Date().toISOString(),
      format: 'OptiQRoute-v1'
    };
    
    return exportData;
  }

  // Save to localStorage
  saveToLocalStorage() {
    try {
      localStorage.setItem('optiqroute_locations', JSON.stringify(this.locations));
    } catch (error) {
      console.warn('Failed to save locations to localStorage:', error);
    }
  }

  // Load from localStorage
  loadFromLocalStorage() {
    try {
      const saved = localStorage.getItem('optiqroute_locations');
      if (saved) {
        this.locations = JSON.parse(saved);
        this.notifyListeners();
        return true;
      }
    } catch (error) {
      console.warn('Failed to load locations from localStorage:', error);
    }
    return false;
  }

  // Clear all data
  clear() {
    this.locations = {
      depot: null,
      customers: []
    };
    this.notifyListeners();
    this.saveToLocalStorage();
  }

  // Get validation status
  getValidationStatus() {
    const locations = this.getAllLocations();
    const validCustomers = locations.customers.filter(c => c && c.coordinates);
    
    return {
      hasDepot: !!locations.depot,
      customerCount: validCustomers.length,
      isValid: !!locations.depot && validCustomers.length >= 2,
      missingDepot: !locations.depot,
      insufficientCustomers: validCustomers.length < 2
    };
  }
}

// Create singleton instance
const locationStorage = new LocationStorageService();

export default locationStorage;
