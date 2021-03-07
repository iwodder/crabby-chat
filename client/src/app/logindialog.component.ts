import { Component } from "@angular/core";
import { MatDialogRef } from "@angular/material/dialog";
import { ChatService } from "src/services/chat.service";

@Component({
    selector: "dialog-b",
    templateUrl: './logindialog.component.html',
  })

  export class LoginDialogComponent {
    public name! : string;
    constructor( public dialogRef: MatDialogRef<LoginDialogComponent>, private cs: ChatService){}

    public joinChatRoom(){
        if(this.name != '' && this.name){
            this.cs.joinChatRoom({name: this.name})
            this.dialogRef.close();
        }
    }
}
