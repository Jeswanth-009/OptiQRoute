# OptiQRoute

A quantum-enhanced route optimization application that uses both classical and quantum algorithms to find optimal delivery routes.

## Features

- **Dual Algorithm Support**: Choose between classical and quantum optimization algorithms
- **Interactive Map**: Visual route planning with Leaflet maps centered on Visakhapatnam, India
- **Real-time Analytics**: Performance metrics and historical trend analysis
- **Responsive Design**: Modern, clean interface that works on different screen sizes
- **Route Visualization**: Compare classical vs quantum optimized routes side by side

## Tech Stack

### Frontend
- **React 19.1.1**: Modern JavaScript library for building user interfaces
- **Leaflet & React-Leaflet**: Interactive mapping library for route visualization
- **Custom CSS**: Responsive design without external frameworks

### Backend
- **Python Flask**: Lightweight web framework (app.py)
- **Route Optimization Algorithms**: Both classical and quantum implementations

## Getting Started

### Prerequisites
- Node.js (version 14 or higher)
- Python 3.x
- npm or yarn

### Installation

1. Clone the repository:
```bash
git clone https://github.com/Jeswanth-009/OptiQRoute.git
cd OptiQRoute
```

2. Install frontend dependencies:
```bash
cd optiqroute-frontend
npm install
```

3. Install backend dependencies:
```bash
pip install flask
```

### Running the Application

1. Start the frontend development server:
```bash
cd optiqroute-frontend
npm start
```

2. Start the backend server:
```bash
python app.py
```

The application will be available at `http://localhost:3000`

## Usage

### Quantum Mode
1. Select the "Quantum" tab in the input panel
2. Enter a start location
3. Add one or more delivery points
4. Set the number of vehicles
5. Optionally add constraints
6. Click "Optimize (Quantum)" to generate the optimized route

### Classical Mode
1. Select the "Classical" tab in the input panel
2. Enter from and to locations
3. Set the number of vehicles
4. Optionally add constraints
5. Click "Optimize (Classical)" to generate the route

## Project Structure

```
OptiQRoute/
├── app.py                      # Flask backend server
├── cache/                      # Cache directory for optimization results
├── optiqroute-frontend/        # React frontend application
│   ├── src/
│   │   ├── App.js             # Main application component
│   │   ├── App.css            # Application styles
│   │   └── ...
│   ├── public/                # Static assets
│   └── package.json           # Frontend dependencies
└── README.md                  # Project documentation
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with modern web technologies
- Quantum optimization algorithms for enhanced performance
- Interactive mapping for better user experience
