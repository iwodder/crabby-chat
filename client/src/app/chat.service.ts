import { Injectable } from '@angular/core';
import { from } from 'rxjs';

@Injectable({
  providedIn: 'root'
})

export class ChatService {
  private socket: WebSocket | undefined;
  private chatUrl = 'ws://localhost:8080/room/';
  private postUrl = 'http://localhost:8000/room/';
  private defaultRoom = 'defaultroom';

  constructor() {
    this.createChatRoom(this.defaultRoom);
    this.setSocket(this.chatUrl + this.defaultRoom);
  //  this.sendMessage('TheUser', 'I\'m here!');
  }

  public sendMessage(name: string, message: string): void {
    console.log('Send message: ' + message);
    this.socket?.send(message);
  }

  public createChatRoom(room: string): void{
    console.log('Create new room: ' + this.postUrl + room);
    const test =  from(
      fetch(
        this.postUrl + room, // the url you are trying to access
        {
          headers: {
            'Content-Type': 'text/plain',
          },
          method: 'Post', // GET, POST, PUT, DELETE
          mode: 'no-cors' // the most important option
        }
      )).subscribe(data => {
        console.log(data);
    });
  }

  public setSocket(room: string): void{
    const newLocal = 'Use new room: ';
    console.log(newLocal + room);
    this.socket = new WebSocket(this.chatUrl + room);
    this.socket.onmessage = (msg => {
      console.log('Received message' + msg.data);
    });
  }
}

