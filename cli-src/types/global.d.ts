declare global {
  interface Window {
    _startTime: number;
    _mouseLog: { t: number; x: number; y: number; clicked: boolean }[];
    [key: string]: any;
  }
}

export {};
