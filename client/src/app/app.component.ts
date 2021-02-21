import { Component } from '@angular/core';
import { ChatService } from './chat.service';

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

  constructor(chatService: ChatService) { this.chatService = chatService; }

  public sendMessage(): void {
    this.chatService.sendMessage(this.authorName, this.authorText);
    this.authorText = '';
  }

  public createChatRoom(): void{
    this.chatService.createChatRoom(this.roomName);
    this.chatRoomList.push(this.roomName);
  }

  public setChatRoom(): void{
    this.chatService.setSocket('');
  }
}
