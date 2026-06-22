import { Component, ReactNode } from "react";

interface Props { children: ReactNode; }
interface State { error: Error | null; }

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  render() {
    if (this.state.error) {
      return (
        <div style={{
          height: "100vh", display: "flex", flexDirection: "column",
          alignItems: "center", justifyContent: "center", gap: 16,
          background: "#0d0d0d", color: "#ece8e1", fontFamily: "monospace",
          padding: 32,
        }}>
          <div style={{ color: "#FF4655", fontSize: 13, fontWeight: 700, letterSpacing: "0.1em", textTransform: "uppercase" }}>
            Something went wrong
          </div>
          <div style={{
            fontSize: 11, color: "#555", background: "#161616",
            border: "1px solid #2c2c2c", padding: "12px 16px",
            maxWidth: 500, wordBreak: "break-all", lineHeight: 1.6,
          }}>
            {this.state.error.message}
          </div>
          <button
            onClick={() => window.location.reload()}
            style={{
              padding: "8px 20px", background: "#FF4655", color: "#fff",
              border: "none", cursor: "pointer", fontSize: 11,
              fontWeight: 700, letterSpacing: "0.1em", textTransform: "uppercase",
            }}
          >
            Reload App
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
