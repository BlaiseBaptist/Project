void setup() {
  // put your setup code here, to run once:
  Serial.begin(9600);
}

void loop() {
  // put your main code here, to run repeatedly:

  Serial.write(0b00000000);
  Serial.write(0b00000000);
  Serial.write(0b11111110);
  Serial.write(0b00000000);

  Serial.flush();
}
