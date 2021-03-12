import { Component, Inject, OnInit, ViewChild } from '@angular/core';
import { ChatService } from '../services/chat.service';
import { MatDialog, MatDialogConfig } from '@angular/material/dialog';
import { LoginDialogComponent } from './logindialog.component';
import { Console } from 'console';
import { MatSelect } from '@angular/material/select';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['../styles/app.component.css', '../styles/app.component.scss'],
})
export class AppComponent {
  messageList: any[] = [{from: "Xavier", msg: "Hello"},{from: "Rafael", msg: "Good morning, How is everyone?"},{from:"Jaylen", msg: "Good morning Ian and Amy"},
  {from: "Amy", msg: "CrabbyChat is so secure and private. I feel comfortable that my data isn't being sold to companies"}, 
  {from: "Ian", msg: "Yeah. Our architecture also enhances security using federated identitity management, using HTTPS, and encryption to ensure our data isn't obtained by hackers."}];
  chatroomSelect = 'Baseball';
  chatRoomList: string[] = [];
  authorText = '';
  roomName = '';
  joined = false;
  title = "Example Angular 10 Material Dialog";
  isDarkTheme: boolean | undefined;
  currentUser= 'Amy';
  
  constructor(private cs: ChatService, private matDialog: MatDialog) {
    this.openDialog(); this.isDarkTheme = true
  }
  
  async ngAfterViewInit() {
    let chatroom = this.cs.getRooms();
    this.chatRoomList = await this.cs.getRooms();
}

  public sendMessage(): void {
    if (this.authorText !== ''){
      this.cs.sendMessage( this.authorText);
      this.authorText = '';
    }
    else{
      console.log('You should say something...');
    }
  }

  public async createChatRoom(): Promise<void>{
    this.cs.createChatRoom(this.roomName);
    this.chatRoomList.push(this.roomName);
    this.roomName = "";
    this.chatRoomList = await this.cs.getRooms();
  }

  public setChatRoom(): void{

  }

  openDialog() {
    this.matDialog.open(LoginDialogComponent, { disableClose: true });
  }
}
