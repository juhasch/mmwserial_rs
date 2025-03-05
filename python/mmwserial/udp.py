"""UDP reader interface for mmwserial."""

from typing import List, Optional
from .mmwserial import UDPReader as _UDPReader

class UDPReader:
    """UDP reader for reading frames from a UDP socket."""
    
    def __init__(self, interface: str, port: int, frame_size: int, timeout_ms: int = 1000):
        """Initialize UDP reader.
        
        Args:
            interface: Network interface to bind to (e.g., "0.0.0.0" for all interfaces)
            port: UDP port to listen on
            frame_size: Expected size of each frame in bytes
            timeout_ms: Timeout in milliseconds for each read operation
        """
        self._reader = _UDPReader(interface, port, frame_size, timeout_ms)
    
    def read_frame(self) -> Optional[bytes]:
        """Read a single frame from the UDP socket.
        
        Returns:
            Frame data as bytes, or None if timeout occurred
        """
        try:
            return self._reader.read_frame()
        except IOError as e:
            if "Timeout" in str(e):
                return None
            raise
    
    def read_frames(self, num_frames: int) -> List[bytes]:
        """Read multiple frames from the UDP socket.
        
        Args:
            num_frames: Number of frames to read
            
        Returns:
            List of frame data as bytes. May be shorter than num_frames if timeout occurred.
        """
        return self._reader.read_frames(num_frames) 