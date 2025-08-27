import { useMapEvents } from 'react-leaflet';

const MapClickHandler = ({ onMapClick }) => {
  useMapEvents({
    click: (e) => {
      onMapClick(e);
    }
  });

  return null;
};

export default MapClickHandler;
