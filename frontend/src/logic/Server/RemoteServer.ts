import type { Server, ServerConnMsg } from "@/logic/Server";

export class RemoteServer implements Server {
  private httpUrl: string;
  private wsUrl: string;
  private socket?: MyWebSocket;
  private reconnectFn?: (status: string) => void;

  constructor(worldId: string) {
    this.httpUrl = `${import.meta.env.VITE_HTTP_URL}/worlds/${worldId}`;
    this.wsUrl = `${import.meta.env.VITE_WS_URL}/worlds/${worldId}`;

    log(`httpUrl = ${this.httpUrl}`);
    log(`wsUrl = ${this.wsUrl}`);
  }

  async join(botId?: string): Promise<ReadableStream<ServerConnMsg>> {
    log("join()", botId);

    const socket = new MyWebSocket(
      botId == null ? `${this.wsUrl}` : `${this.wsUrl}/bots/${botId}`,
    );

    const msgs = new ReadableStream({
      start(ctrl) {
        socket.onmessage = (event) => {
          ctrl.enqueue(JSON.parse(event.data));
        };
      },

      cancel() {
        socket.close();
      },
    });

    await socket.connect();

    const reconnect = async () => {
      if (socket.isClosing) {
        return;
      }

      log("reconnecting");

      if (this.reconnectFn) {
        this.reconnectFn("reconnecting");
      }

      while (true) {
        try {
          await this.join(botId);
          break;
        } catch (err) {
          await new Promise((resolve) => {
            setTimeout(resolve, 250);
          });
        }
      }

      log("reconnected");

      if (this.reconnectFn) {
        this.reconnectFn("connected");
      }
    };

    socket.onclose = reconnect;
    socket.onerror = reconnect;

    this.socket = socket;

    log("ready");

    return msgs;
  }

  async close(): Promise<void> {
    log("close()");

    if (this.socket) {
      this.socket.close();
    }
  }

  async uploadBot(src: File): Promise<{ id: string }> {
    log("uploadBot()");

    const response = await fetch(`${this.httpUrl}/bots`, {
      method: "POST",
      body: src,
    });

    if (response.status == 200) {
      return await response.json();
    } else {
      throw await response.text();
    }
  }

  onReconnect(f: (status: string) => void): void {
    this.reconnectFn = f;
  }
}

class MyWebSocket extends WebSocket {
  isClosing: boolean = false;

  connect(): Promise<void> {
    log("connecting");

    return new Promise((resolve, reject) => {
      this.onopen = () => {
        resolve(null);
      };

      this.onclose = () => {
        // Prevent the other handler from firing `reject()` again
        this.onerror = null;

        reject();
      };

      this.onerror = () => {
        // Prevent the other handler from firing `reject()` again
        this.onclose = null;

        reject();
      };
    });
  }

  close(): void {
    log("closing");

    this.isClosing = true;
    super.close();
  }
}

function log(...data: any[]) {
  console.log("[remote-server]", ...data);
}
