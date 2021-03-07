import { webSocket, WebSocketSubject} from 'rxjs/webSocket';
import { catchError, tap, switchAll } from 'rxjs/operators';
import { Injectable} from '@angular/core';
import { EMPTY, Subject } from 'rxjs';

@Injectable({
    providedIn: 'root'
  })

  
export class WebSocketService {
    private socket: WebSocketSubject<any> | undefined;
    private messagesSubject$ = new Subject();
    
    public connect(chat: string): void {
    
      if (!this.socket || this.socket.closed) {
        this.socket = this.getNewWebSocket(chat);
        this.socket.subscribe(
            (msg) => console.log('message received: ' + msg),
            () => console.log('complete')
          );
      }
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