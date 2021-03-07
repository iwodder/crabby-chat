import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { map } from "rxjs/operators";
import { Message } from '../objects/message';
import { User } from '../objects/user';
import { WebSocketService } from './websocket.service';

@Injectable({
  providedIn: 'root'
})

export class ChatService {
 // private socket: WebSocket | undefined;
  private defaultRoom = 'defaultroom';
  private chatUrl = 'ws://localhost:8080/room/';
  private user = '';
  private headers = new HttpHeaders().set('Content-Type', 'application/json');

  constructor(private http: HttpClient, private ws: WebSocketService) {
    this.createChatRoom(this.defaultRoom);
  }

  public sendMessage(text: string): void {
    try{
      this.ws.sendMessage({msg : text, from: this.user});
    }
    catch(e){
      console.log('error');
    }
  }

  public async createChatRoom(room: string){
    try{
      let available = await this.isRoomAvailable(room);
      if(available){
        console.log("Create new room: " + room);
        let result = this.http.post('room/' + room, { headers: this.headers})
        .subscribe( resp => {console.log('Create room: ' + room)});
      }
    }
    catch(e){
      console.log(e);
    }
    this.ws.connect(this.chatUrl + room);
  }

  private async isRoomAvailable(room:string){
    console.log('Check whether room exists');
    let available = false;;
    let result =  await this.http.get('room/available?names=' + room, { headers: this.headers}).toPromise().then((data: any) => {
      available = data[0]['available'];
    })
    console.log('Is this room available: ' + available);
    return available;

  }

  public joinChatRoom(user: User): void{
    console.log(user.name + ' has joined the room');
    this.user = user.name;
    this.ws.sendMessage(user);
  }
}

