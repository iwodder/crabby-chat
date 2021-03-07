import { Component, Inject, OnInit } from '@angular/core';
import { ChatService } from '../services/chat.service';
import { MatDialog, MatDialogConfig } from '@angular/material/dialog';
import { LoginDialogComponent } from './logindialog.component';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['../styles/app.component.css', '../styles/app.component.scss'],
})
export class AppComponent {
  messageList: string[] = [];
  chatRoomList: string[] = [];
  authorText = '';
  roomName = '';
  joined = false;
  title = "Example Angular 10 Material Dialog";
  isDarkTheme: boolean | undefined;
  
  constructor(private cs: ChatService, private matDialog: MatDialog) {this.openDialog(); this.isDarkTheme = true}

  public sendMessage(): void {
    if (this.authorText !== ''){
      this.cs.sendMessage( this.authorText);
      this.authorText = '';
    }
    else{
      console.log('You should say something...');
    }
  }

  public createChatRoom(): void{
    this.cs.createChatRoom(this.roomName);
    this.chatRoomList.push(this.roomName);
  }

  public setChatRoom(): void{
   // this.chatService.setSocket('');
  }

  openDialog() {
    this.matDialog.open(LoginDialogComponent, { disableClose: true });
  }
}
