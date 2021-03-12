import { webSocket, WebSocketSubject} from 'rxjs/webSocket';
import { Injectable} from '@angular/core';
import { of, Subject } from 'rxjs';
import { NULL_EXPR } from '@angular/compiler/src/output/output_ast';

@Injectable({
    providedIn: 'root'
  })

  
export class WebSocketService {
    private socket!: WebSocketSubject<any> | null;
    
    public connect(chat: string): WebSocketSubject<any> | null{    
      if (!this.socket || this.socket.closed) {
        return this.getNewWebSocket(chat);
      }
      return null;
    }
    
    private getNewWebSocket(chat: string) {
      return webSocket(chat);
    }
    sendMessage(msg: any) {
        this.socket?.next(JSON.stringify(msg));
    }

    close() {
      this.socket?.complete(); 
    }
}