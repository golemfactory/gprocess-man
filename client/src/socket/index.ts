export interface Socket {
    onClose(callback: () => void): void;
    onData(callback: (data: Uint8Array) => void): void;
    onError(callback: (error: Error) => void): void;
    write(data: Uint8Array): void;
    close(): void;
}
