import { Component } from '@angular/core';
import { ChatService } from './chat.service';
import { Message } from './message';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css'],
})
export class AppComponent {
  private chatService: ChatService;
  messageList: string[] = [];
  chatRoomList: string[] = [];
  authorText = '';
  authorName = '';
  roomName = '';
  joined = false;

  constructor(chatService: ChatService) { this.chatService = chatService; }

  public sendMessage(): void {
    if (this.authorText !== ''){
      this.chatService.sendMessage({msg: this.authorText, from: this.authorName});
      this.authorText = '';
    }
    else{
      console.log('You should say something...');
    }
  }

  public createChatRoom(): void{
    this.chatService.createChatRoom(this.roomName);
    this.chatRoomList.push(this.roomName);
  }

  public setChatRoom(): void{
    this.chatService.setSocket('');
  }

  public joinChatRoom(){
    this.chatService.joinChatRoom({name: this.authorName})
  }
}
