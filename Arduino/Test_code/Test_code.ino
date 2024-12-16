void setup() {
  // put your setup code here, to run once:
  Serial.begin(9600);

}
  int C = 0;
void loop() {
  // put your main code here, to run repeatedly:
  C += 1;
  Serial.write(0b00000000);
  Serial.write(0b00000000);
  Serial.write(0b00000000);
  Serial.write(C);


  Serial.flush();
}
