/*
Author: Michael

2017/10/28

*/

/* [basic parameters] */

//How wide is the tape?
tapeWidth=8; // [8, 12, 16, 24, 36, 48, 72]
//Optimize for Plastic or Paper tape? Plastic tapes fit in Paper, but not reverse. Plastic tapes may be too loose in paper feeder.
tapeHeight=0.9; // [0.9:paper,0.3:plastic]
//How many feeder to print ganged
numberOfFeeders=2; // [1:1:20]
//Overall length of feeder?
feederLength=180;
//Height of tape's bottom side above bed
tapeLayerHeight=22.8;
//Bank ID: To identify the feeder in OpenPnP unique IDs for each bank are built and embossed into the ganged feeder. -1: no identifier.
bankID=1; //[-1:1:9]
//Change the starting letter according to ASCII code: 65 -> 'A', 75 -> 'K', 85 -> 'U' and so on.
startingLetter=65; //[65:1:90]
//diameter of pockets on the side to put a magnet in
magnetDiameter=6;
//height of the pocket for the magnet
magnetHeight=3;
//How many magnet pockets in the right side?
magnetPocketsRightSide=3;
//How many magnet pockets on the left side?
magnetPocketsLeftSide=4;

/* [advanced] */

//controls the force to keep the tape in place. lower value higher force and though friction
tapeClearance=-0.3;     // [-0.5:0.05:0.5]
bodyHeight=6;
tapeSupportHoleSide=2.8;
tapeSupportNonHoleSide=0.9;

//Separation from the magnet pocket to the edge, right side
separationRightSide=24.5;
//Separation from the magnet pocket to the edge, left side
separationLeftSide=7.5;

// Change this value to get a thicker space under the magnet
layerBelow=0.35;

/* [expert] */
//higher values make the left arm stronger
additionalWidth=3;
topFinishingLayer=0.3;
tapeGuideUpperOverhang=0.4;
//lower values make the spring smaller thus less force on tape, should be slightly less than a multiple of extrusion-width
springWidth=1.3;
springSkew=1.2;
//if two tapeloaded lanes touch each other raise this value a little
springClearance=0.4;

overallWidth=tapeWidth+additionalWidth;
overallHeight=tapeLayerHeight+tapeHeight+tapeGuideUpperOverhang+topFinishingLayer;
tapeXcenter=(overallWidth/2)+tapeClearance/2;



//make the feeders
gang_feeder();


// customizer specific stuff
// preview[view:north west, tilt:top diagonal]


module gang_feeder() {

    difference() {
        union() {


            //stack up feeders
            for(i=[0:1:numberOfFeeders-1]) {
                translate([i*(tapeWidth+additionalWidth),0,0])
                    feeder_body(i);
            }


        }

        //magnet pockets
        //echo((feederLength-2*separationRightSide)/(magnetPocketsRightSide-1))
        for(j=[0:1:magnetPocketsRightSide-1]) {
            //magnet pockets right side
            translate([0,j*((feederLength-2*separationRightSide)/(magnetPocketsRightSide-1))+separationRightSide,0]) {
                translate([(numberOfFeeders)*(tapeWidth+additionalWidth),0,0 ]) {
                    //magnetic_fixation_pocket(0,feederLength/2);
                    magnetic_fixation_pocket();
                }
            }

        }

        //echo((feederLength-2*separationLeftSide)/(magnetPocketsLeftSide-1))
        for(j=[0:1:magnetPocketsLeftSide-1]) {
            //magnet pockets left side
            translate([0,j*((feederLength-2*separationLeftSide)/(magnetPocketsLeftSide-1))+separationLeftSide,0]) {
                rotate([0,0,180])
                    //magnetic_fixation_pocket(0,feederLength/2);
                    magnetic_fixation_pocket();
            }
        }
    }
}

module feeder_cover() {
    cube([2,feederLength,overallHeight]);
}

module feeder_body(feederNo) {
    translate([0,feederLength,0]) {
        rotate([90,0,0]) {
            difference() {

                //main form
                linear_extrude(feederLength) {
                    polygon(points=[
                        //base
                        [0,0],
                        [overallWidth,0],

                        //right arm way up ("spring", outer part)
                        [overallWidth,bodyHeight*0.8],
                        [overallWidth-springSkew,tapeLayerHeight-3],
                        [overallWidth-springClearance,tapeLayerHeight],

                        //right arm tape guide
                        [overallWidth-springClearance,overallHeight],
                        [tapeXcenter+tapeWidth/2+tapeClearance-tapeGuideUpperOverhang,overallHeight],
                        [tapeXcenter+tapeWidth/2+tapeClearance-tapeGuideUpperOverhang,tapeLayerHeight+tapeHeight+tapeGuideUpperOverhang],
                        [tapeXcenter+tapeWidth/2+tapeClearance,tapeLayerHeight+tapeHeight],
                        [tapeXcenter+tapeWidth/2+tapeClearance,tapeLayerHeight],
                        [tapeXcenter+tapeWidth/2+tapeClearance-tapeSupportHoleSide,tapeLayerHeight],
                        [tapeXcenter+tapeWidth/2+tapeClearance-tapeSupportHoleSide,tapeLayerHeight-0.6],

                        //right arm way down ("spring", inner part)
                        [overallWidth-springSkew-springWidth,tapeLayerHeight-3],
                        [overallWidth-springWidth,bodyHeight*0.8],
                        [overallWidth-springWidth-1,bodyHeight],

                        //base (inner part)
                        [tapeXcenter-tapeWidth/2+tapeSupportNonHoleSide,bodyHeight],

                        //left arm up (inner part)
                        [tapeXcenter-tapeWidth/2+tapeSupportNonHoleSide,tapeLayerHeight],

                        //left arm tape guide
                        [tapeXcenter-tapeWidth/2,tapeLayerHeight],
                        [tapeXcenter-tapeWidth/2,tapeLayerHeight+tapeHeight],
                        [tapeXcenter-tapeWidth/2+tapeGuideUpperOverhang,tapeLayerHeight+tapeHeight+tapeGuideUpperOverhang],
                        [tapeXcenter-tapeWidth/2+tapeGuideUpperOverhang,overallHeight],

                        //left arm down (outer part)
                        [0,overallHeight],
                        [0,0]

                    ]);
                }

                //direction of travel while picking with OpenPnP
                if(feederLength>=100) {
                    for (i=[0:1:3]) {
                        translate([additionalWidth+3/2+0.5,bodyHeight+0.1,feederLength-25-(i*6)])
                            rotate([90,90,0])
                                linear_extrude(1)
                                    circle(3,$fn=3);
                    }
                }

                //4 identification marks
                translate([additionalWidth,bodyHeight-0.9,feederLength-2])
                    rotate([90,90,180])
                        identification_mark(feederNo,"left","top");

                translate([additionalWidth,bodyHeight-0.9,2])
                    rotate([90,90,180])
                        identification_mark(feederNo,"right","top");

                translate([tapeXcenter,bodyHeight-0.9,feederLength-0.9])
                    rotate([0,0,0])
                        identification_mark(feederNo,"center","top");

                translate([tapeXcenter,bodyHeight-0.9,0.9])
                    rotate([0,180,0])
                        identification_mark(feederNo,"center","top");

                //reference hole
                translate([tapeXcenter+tapeWidth/2+tapeClearance-1.75,tapeLayerHeight,feederLength-4])
                    rotate([90,90,0])
                        cylinder(h=0.6,d=1.4,center=false,$fn=20);

                //3 registration points (for magnets, bolts or to screw from top)
                //bottom_fixation(feederLength/2);
                bottom_fixation(17);
                bottom_fixation(feederLength-17);


            }
        }
    }
}

module identification_mark(feederNo,_halign,_valign) {

    if(bankID!=-1) {
        linear_extrude(height=.91) {
            text( str(bankID, chr(feederNo+startingLetter) ),font=":style=Bold", size=4, valign=_valign, halign=_halign);
        }
    }

}

module magnetic_fixation_pocket() {
    // change 'layerBelow' to get a thicker space under the magnet, 2 layers is ok, check your slicer's settings and preview!. Default: layerBelow=0.25;
    magnetInset=1;
    magnetDiameterOversizedFor3dPrinting=magnetDiameter+0.2;

    translate([0,0,layerBelow]) {
            union() {
                translate([-(magnetDiameterOversizedFor3dPrinting)/2-magnetInset,0,0])
                    cylinder(d=magnetDiameterOversizedFor3dPrinting,h=magnetHeight+0.3,$fn=20);

                hull() {
                    translate([-(magnetDiameterOversizedFor3dPrinting)/2-magnetInset,0,0])
                        cylinder(d=magnetDiameterOversizedFor3dPrinting-1.4,h=magnetHeight+0.3,$fn=20);
                    translate([0,0,(magnetHeight+0.3)/2])
                        cube([0.1,magnetDiameterOversizedFor3dPrinting+0.4,magnetHeight+0.3],center=true);
                }

                translate([-(magnetDiameterOversizedFor3dPrinting)/2-magnetInset,0,0]) {
                    difference() {
                        cylinder(d=magnetDiameterOversizedFor3dPrinting+3,h=magnetHeight+0.3,$fn=20);
                        cylinder(d=magnetDiameterOversizedFor3dPrinting+2,h=magnetHeight+0.3,$fn=20);
                    }
                }
            }
        translate([-(magnetDiameterOversizedFor3dPrinting-magnetInset+1)/2-magnetInset,0,0])
            cube([magnetDiameterOversizedFor3dPrinting,1,1],center=true);
    }
}

module bottom_fixation(pos_y) {
    layerForBridging=0.3;
    cutoutbelow=3.5;
    union() {
        translate([tapeXcenter,bodyHeight-1,pos_y])
                rotate([-90,0,0])
                    cylinder(h = 2.1, r=6.0/2, $fn=20);

        translate([tapeXcenter,-0.1,pos_y])
                rotate([-90,0,0])
                    cylinder(h = bodyHeight+1, r=3.5/2, $fn=20);

        //old pocket below feeder for magnet
        *translate([tapeXcenter,cutoutbelow,pos_y])
                rotate([90,0,0])
                    cylinder(h = 10, r=6.0/2, $fn=20);

        //chamfer
        *translate([tapeXcenter,0.3,pos_y])
                rotate([90,0,0])
                    cylinder(h = 0.3, r1=6.0/2, r2=6.3/2, $fn=20);
    }
}
