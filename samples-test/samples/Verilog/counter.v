`timescale 1ns / 1ps

// A simple parameterized up-counter with asynchronous reset.
module counter #(
    parameter WIDTH = 8
) (
    input  wire             clk,
    input  wire             rst_n,
    output reg  [WIDTH-1:0] count
);

  always @(posedge clk or negedge rst_n) begin
    if (!rst_n) count <= {WIDTH{1'b0}};
    else count <= count + 1'b1;
  end

endmodule
