variable "app-name" {
  type        = string
  default     = "rust-test"
  description = "Name of the app"
}

variable "tags" {
  type = map(any)
  default = {
    NAME = "rust-test"
  }
}
