import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { map } from "rxjs/operators";
import { Message } from './message';
import { User } from './user';

@Injectable({
  providedIn: 'root'
})

export class ChatService {
  private socket: WebSocket | undefined;
  private defaultRoom = 'defaultroom';
  private chatUrl = 'ws://localhost:8080/room/';
  private headers = new HttpHeaders().set('Content-Type', 'application/json');

  constructor(private http: HttpClient) {
    this.createChatRoom(this.defaultRoom);
    this.setSocket(this.defaultRoom);
  }

  public sendMessage(msg: Message): void {
    console.log('Send message: ' + msg.msg + ' From: ' + msg.from);
    try{
      this.socket?.send(JSON.stringify(msg));
    }
    catch(e){
      console.log('error');
    }
  }

  public async createChatRoom(room: string){
    try{
      let available = await this.isRoomAvailable(room);
      console.log(available);
      if(available){
        console.log("Create new room: " + room);
        let result = this.http.post('room/' + room, { headers: this.headers})
        .subscribe( resp => {console.log('Create room: ' + room)});
      }
    }
    catch(e){
      console.log(e);
    }
  }

  private async isRoomAvailable(room:string){
    console.log('Check whether room exists');
    let available = false;;
    let result =  await this.http.get('room/available?names=' + room, { headers: this.headers}).toPromise().then((data: any) => {
      console.log(data[0]['available']);
      available = data[0]['available'];
    })
    console.log('Is this room available: ' + available);
    return available;

  }

  public setSocket(room: string): void{
    this.socket = new WebSocket(this.chatUrl + room);
    this.socket.onerror = (e => console.log(e));
    this.socket.onmessage = (msg => console.log('Received message' + msg.data));
  }

  public joinChatRoom(user: User): void{
    console.log(user.name + ' has joined the room');
    this.socket?.send(JSON.stringify(user));
  }
}

