#include <Ethernet.h>
#include "arduino_secrets.h"

EthernetClient mqttbrokerclient;
byte CLIENT_MAC[] = { 0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0xED };
char CLIENT_IP[] = "192.168.50.45";

//IPAddress mqttBrokerIP(192, 168, 50, 24);  // (Marcus Computer)
IPAddress mqttBrokerIP(85, 83, 170, 136);  // (HACK THIS Computer)
const int mqttBrokerPort = 1883;  

//Delivery parcel 1 LED pins
const int ledparcel1CentralPin = 8;
const int ledparcel1InRoutePin = 9;
const int ledparcel1DeliveredPin = A0;//10;

//Delivery parcel 1 button pins
const int parcel1CentralPin = 2;  
const int parcel1InRoutePin = 3;  
const int parcel1DeliveredPin = 4;  

//Delivery parcel 2 LED pins
const int ledparcel2CentralPin = A1;//11;
const int ledparcel2InRoutePin = A2;//12;
const int ledparcel2DeliveredPin = A3;//13;

//Delivery parcel 2 button pins
const int parcel2CentralPin = 5;  
const int parcel2InRoutePin = 6;  
const int parcel2DeliveredPin = 7;  


//To keep track of the state of the buttons for parcel 1
int parcel1CentralCurrentState = 0;
int parcel1CentralPreviousState = 0;  
int parcel1InRouteCurrentState = 0; 
int parcel1InRoutePreviousState = 0; 
int parcel1DeliveredCurrentState = 0;
int parcel1DeliveredPreviousState = 0;  

//To keep track of the state of the buttons for parcel 2
int parcel2CentralCurrentState = 0;
int parcel2CentralPreviousState = 0;  
int parcel2InRouteCurrentState = 0; 
int parcel2InRoutePreviousState = 0; 
int parcel2DeliveredCurrentState = 0;
int parcel2DeliveredPreviousState = 0;  

int parcel1DeliveryState = 0;
int parcel1DeliveryMessageState = 0;

int parcel2DeliveryState = 0;
int parcel2DeliveryMessageState = 0;

bool mqttConnected = false;

void setup() {

  Serial.begin(9600);
  // initialize the LED pins as an output:
  pinMode(ledparcel1CentralPin, OUTPUT);
  pinMode(ledparcel1InRoutePin, OUTPUT);
  pinMode(ledparcel1DeliveredPin, OUTPUT);
  // initialize the button pins as an Input:
  pinMode(parcel1CentralPin, INPUT);
  pinMode(parcel1InRoutePin, INPUT);
  pinMode(parcel1DeliveredPin, INPUT);
  // initialize the LED pins as an output:
  pinMode(ledparcel2CentralPin, OUTPUT);
  pinMode(ledparcel2InRoutePin, OUTPUT);
  pinMode(ledparcel2DeliveredPin, OUTPUT);  
  // initialize the button pins as an Input:
  pinMode(parcel2CentralPin, INPUT);
  pinMode(parcel2InRoutePin, INPUT);
  pinMode(parcel2DeliveredPin, INPUT);

  while (!Serial) {
    ;
  }

  while (Ethernet.begin(CLIENT_MAC, CLIENT_IP) != 1)
  {
    Serial.println("Error getting IP address, trying again...");
    delay(5000);
  }
  Serial.println("Internet connected");
  Serial.print("My IP address: ");
  Serial.println(Ethernet.localIP());
}

void loop() {
  parcel1CentralCurrentState = digitalRead(parcel1CentralPin);
  parcel1InRouteCurrentState = digitalRead(parcel1InRoutePin);
  parcel1DeliveredCurrentState = digitalRead(parcel1DeliveredPin);
  
  parcel2CentralCurrentState = digitalRead(parcel2CentralPin);
  parcel2InRouteCurrentState = digitalRead(parcel2InRoutePin);
  parcel2DeliveredCurrentState = digitalRead(parcel2DeliveredPin);

  if (!mqttConnected || !mqttbrokerclient.connected()) {
    // If MQTT connection is lost, attempt to reconnect
    Serial.println("Connection to MQTT broker lost. Reconnecting...");
    delay(3000);
    mqttConnected = connectToMQTTBroker();
  }

//**********************************PARCEL 1 BUTTON PRESSES***********************************//
  //Toggle of the first button of parcel 1
  if (parcel1CentralCurrentState != parcel1CentralPreviousState)
  {
    if(parcel1CentralCurrentState == HIGH && parcel1DeliveryMessageState != 1)
    {
      parcel1DeliveryState = 1;      
    }
    if(parcel1CentralCurrentState == HIGH && parcel1DeliveryMessageState == 1)
    {
      parcel1DeliveryState = 0;      
    }
  }
  //Toggle of the second button of parcel 1
  if (parcel1InRouteCurrentState != parcel1InRoutePreviousState)
    {
      if(parcel1InRouteCurrentState == HIGH && parcel1DeliveryMessageState != 2)
      {
        parcel1DeliveryState = 2;      
      }
      if(parcel1InRouteCurrentState == HIGH && parcel1DeliveryMessageState == 2)
      {
        parcel1DeliveryState = 0;      
      }
    }
  //Toggle of the third button of parcel 1
  if (parcel1DeliveredCurrentState != parcel1DeliveredPreviousState)
  {
    if(parcel1DeliveredCurrentState == HIGH && parcel1DeliveryMessageState != 3)
    {
      parcel1DeliveryState = 3;      
    }
    if(parcel1DeliveredCurrentState == HIGH && parcel1DeliveryMessageState == 3)
    {
      parcel1DeliveryState = 0;      
    }
  }
//**********************************PARCEL 1  BUTTON PRESSES END*******************************//


//**********************************PARCEL 2 BUTTON PRESSES***********************************//
  //Toggle of the first button of parcel 2
  if (parcel2CentralCurrentState != parcel2CentralPreviousState)
  {
    if(parcel2CentralCurrentState == HIGH && parcel2DeliveryMessageState != 1)
    {
      parcel2DeliveryState = 1;      
    }
    if(parcel2CentralCurrentState == HIGH && parcel2DeliveryMessageState == 1)
    {
      parcel2DeliveryState = 0;      
    }
  }
  //Toggle of the second button of parcel 2
  if (parcel2InRouteCurrentState != parcel2InRoutePreviousState)
    {
      if(parcel2InRouteCurrentState == HIGH && parcel2DeliveryMessageState != 2)
      {
        parcel2DeliveryState = 2;      
      }
      if(parcel2InRouteCurrentState == HIGH && parcel2DeliveryMessageState == 2)
      {
        parcel2DeliveryState = 0;      
      }
    }
  //Toggle of the third button of parcel 2
  if (parcel2DeliveredCurrentState != parcel2DeliveredPreviousState)
  {
    if(parcel2DeliveredCurrentState == HIGH && parcel2DeliveryMessageState != 3)
    {
      parcel2DeliveryState = 3;      
    }
    if(parcel2DeliveredCurrentState == HIGH && parcel2DeliveryMessageState == 3)
    {
      parcel2DeliveryState = 0;      
    }
  }
//**********************************PARCEL 2 BUTTON PRESSES END*******************************//


//**********************************PARCEL 1  MESSAGES AND LIGHT******************************//
  //Sends the clear message for topic 1 to the broker and sets the LED's accordingly.
  if ( parcel1DeliveryState == 0 && parcel1DeliveryMessageState != 0){
    SetDiodes(1, parcel1DeliveryState);
    uint8_t connectPacket[] = {49, 15, 0, 13, 65, 66, 67, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48};
    mqttbrokerclient.write(connectPacket, sizeof(connectPacket));
    parcel1DeliveryMessageState = 0;
    delay(500);
    Serial.println("Parcel 1 Clear message sent");
  }
  
  //Sends the Publish 1 packet for topic 1 to the broker and sets the LED's accordingly.
  if ( parcel1DeliveryState == 1 && parcel1DeliveryMessageState != 1){
    SetDiodes(1, parcel1DeliveryState);
    uint8_t connectPacket[] = {49, 16, 0, 13, 65, 66, 67, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 48};
    mqttbrokerclient.write(connectPacket, sizeof(connectPacket));
    parcel1DeliveryMessageState = 1;
    delay(500);
    Serial.println("Parcel 1 Message 1 sent");
  }

  //Sends the Publish 1 packet for topic 1 to the broker and sets the LED's accordingly.  
  if ( parcel1DeliveryState == 2 && parcel1DeliveryMessageState != 2){
    SetDiodes(1, parcel1DeliveryState);
    uint8_t connectPacket[] = {49, 16, 0, 13, 65, 66, 67, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49};
    mqttbrokerclient.write(connectPacket, sizeof(connectPacket));
    parcel1DeliveryMessageState = 2;
    delay(500);
    Serial.println("Parcel 1 Message 2 sent");
  }

  //Sends the Publish 1 packet for topic 1 to the broker and sets the LED's accordingly.
  if ( parcel1DeliveryState == 3 && parcel1DeliveryMessageState != 3){
    SetDiodes(1, parcel1DeliveryState);
    uint8_t connectPacket[] = {49, 16, 0, 13, 65, 66, 67, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 50};
    mqttbrokerclient.write(connectPacket, sizeof(connectPacket));
    parcel1DeliveryMessageState = 3;
    delay(500);
    Serial.println("Parcel 1 Message 3 sent");
  }
  //**********************************PARCEL 1  MESSAGES AND LIGHT END******************************//


//**********************************PARCEL 2  MESSAGES AND LIGHT**********************************//
  //Sends the clear message for topic 2 to the broker and sets the LED's accordingly.
  if ( parcel2DeliveryState == 0 && parcel2DeliveryMessageState != 0){
    SetDiodes(2, parcel2DeliveryState);
    uint8_t connectPacket[] = {49, 15, 0, 13, 88, 89, 90, 57, 56, 55, 54, 53, 52, 51, 50, 49, 48};
    mqttbrokerclient.write(connectPacket, sizeof(connectPacket));
    parcel2DeliveryMessageState = 0;
    delay(500);
    Serial.println("Parcel 2 Clear message sent");
  }
  
  //Sends the Publish 1 packet for topic 2 to the broker and sets the LED's accordingly.
  if ( parcel2DeliveryState == 1 && parcel2DeliveryMessageState != 1){
    SetDiodes(2, parcel2DeliveryState);
    uint8_t connectPacket[] = {49, 16, 0, 13, 88, 89, 90, 57, 56, 55, 54, 53, 52, 51, 50, 49, 48, 48};
    mqttbrokerclient.write(connectPacket, sizeof(connectPacket));
    parcel2DeliveryMessageState = 1;
    delay(500);
    Serial.println("Parcel 2 Message 1 sent");
  }

  //Sends the Publish 1 packet for topic 2 to the broker and sets the LED's accordingly.  
  if ( parcel2DeliveryState == 2 && parcel2DeliveryMessageState != 2){
    SetDiodes(2, parcel2DeliveryState);
    uint8_t connectPacket[] = {49, 16, 0, 13, 88, 89, 90, 57, 56, 55, 54, 53, 52, 51, 50, 49, 48, 49};
    mqttbrokerclient.write(connectPacket, sizeof(connectPacket));
    parcel2DeliveryMessageState = 2;
    delay(500);
    Serial.println("Parcel 2 Message 2 sent");
  }

  //Sends the Publish 1 packet for topic 2 to the broker and sets the LED's accordingly.
  if ( parcel2DeliveryState == 3 && parcel2DeliveryMessageState != 3){
    SetDiodes(2, parcel2DeliveryState);
    uint8_t connectPacket[] = {49, 16, 0, 13, 88, 89, 90, 57, 56, 55, 54, 53, 52, 51, 50, 49, 48, 50};
    mqttbrokerclient.write(connectPacket, sizeof(connectPacket));
    parcel2DeliveryMessageState = 3;
    delay(500);
    Serial.println("Parcel 2 Message 3 sent");
  }
  //**********************************PARCEL 2  MESSAGES AND LIGHT END******************************//

  //Updates the state of the buttons
  parcel1CentralPreviousState     = parcel1CentralCurrentState;
  parcel1InRoutePreviousState     = parcel1InRouteCurrentState;
  parcel1DeliveredPreviousState   = parcel1DeliveredCurrentState;
  parcel2CentralPreviousState     = parcel2CentralCurrentState;
  parcel2InRoutePreviousState     = parcel2InRouteCurrentState;
  parcel2DeliveredPreviousState   = parcel2DeliveredCurrentState;
}


//Method to set the diodes according to the state of the delivery state
void SetDiodes(int packetNumber, int parcelState){
  switch (packetNumber) {
    case 1:
      switch (parcelState) {
        case 0:
          digitalWrite(ledparcel1CentralPin, LOW);
          digitalWrite(ledparcel1InRoutePin, LOW);
          analogWrite(ledparcel1DeliveredPin, 0);
          break;
        case 1:
          digitalWrite(ledparcel1CentralPin, HIGH);
          digitalWrite(ledparcel1InRoutePin, LOW);
          analogWrite(ledparcel1DeliveredPin, 0);
          break;
        case 2:
          digitalWrite(ledparcel1CentralPin, LOW);
          digitalWrite(ledparcel1InRoutePin, HIGH);
          analogWrite(ledparcel1DeliveredPin, 0);
          break;
        case 3:
        digitalWrite(ledparcel1CentralPin, LOW);
        digitalWrite(ledparcel1InRoutePin, LOW);
        analogWrite(ledparcel1DeliveredPin, 255);
        break;
      }
      break;
    case 2:
      switch (parcelState) {
        case 0:
          analogWrite(ledparcel2CentralPin, 0);
          analogWrite(ledparcel2InRoutePin, 0);
          analogWrite(ledparcel2DeliveredPin, 0);
          break;
        case 1:
          analogWrite(ledparcel2CentralPin, 255);
          analogWrite(ledparcel2InRoutePin, 0);
          analogWrite(ledparcel2DeliveredPin, 0);
          break;
        case 2:
          analogWrite(ledparcel2CentralPin, 0);
          analogWrite(ledparcel2InRoutePin, 255);
          analogWrite(ledparcel2DeliveredPin, 0);
          break;
        case 3:
          analogWrite(ledparcel2CentralPin, 0);
          analogWrite(ledparcel2InRoutePin, 0);
          analogWrite(ledparcel2DeliveredPin, 255);
          break;
      }
  }
}

//If the Arduino loses the connection to the client, it will try to reconnect.
bool connectToMQTTBroker() {
  Serial.println("Attempting to connect to MQTT broker...");
  if (mqttbrokerclient.connect(mqttBrokerIP, mqttBrokerPort)) {
    Serial.println("Connected to MQTT broker");
    uint8_t connectPacket[] = {16, 26, 0, 4, 77, 81, 84, 84, 4, 2, 14, 16, 0, 14, 97, 114, 100, 117, 105, 110, 111, 95, 112, 97, 114, 99, 101, 108};
    mqttbrokerclient.write(connectPacket, sizeof(connectPacket));
    return true;
  } else {
    Serial.println("Failed to connect to MQTT broker. Retrying in 5 seconds...");
    delay(5000); // Wait 5 seconds before retrying
    return false;
  }
}
