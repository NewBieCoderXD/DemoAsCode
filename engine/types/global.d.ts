declare global {
  interface Window {
    __recorder_streamMouseLog?: (entry: Omit<MouseLogEntry, "t">) => void;
  }
}

export {};
