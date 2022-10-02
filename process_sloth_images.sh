#!/usr/bin/env bash
SLOTH_IMAGES=$(ls "$1")

[ ! -d "$1" ] && echo "Directory $1 does not exist" && exit 128

for SLOTH_IMAGE_FILE in $SLOTH_IMAGES; do
  SLOTH_IMAGE_FILE=$(basename -- "$SLOTH_IMAGE_FILE")
  SLOTH_IMAGE_FILE_BASENAME="${SLOTH_IMAGE_FILE%.*}"
  SLOTH_IMAGE_FILE_EXTENSION="${SLOTH_IMAGE_FILE##*.}"
  SLOTH_IMAGE_PATH="$1/$SLOTH_IMAGE_FILE"
  HASHED_NAME=$(echo "$SLOTH_IMAGE_FILE" | sha256sum | awk '{ print $1 }')

  # First make everything jpg
  convert "$SLOTH_IMAGE_PATH" -define jpeg "$1/$HASHED_NAME.jpg"
  rm "$SLOTH_IMAGE_PATH"
  SLOTH_IMAGE_PATH="$1/$HASHED_NAME.jpg"

  IMAGE_DETAILS=$(identify "$SLOTH_IMAGE_PATH")
  RESOLUTION=$(echo "$IMAGE_DETAILS" | awk '{ print $3 }')
  WIDTH=$(echo "$RESOLUTION" | awk -F'x' '{ print $1 }')
  HEIGHT=$(echo "$RESOLUTION" | awk -F'x' '{ print $2 }')
  RATIO=$(echo "$WIDTH / $HEIGHT" | bc -l)

# For now only sort into slightly more vertical, slighty more horizontal and square images,
# maybe expand once more get added
  RATIOS="
1.33333333333333333 4_BY_3
1.0 1_BY_1
0.75 3_BY_4
"

  CLOSEST_RATIO=$(echo "$RATIOS" | awk -v c=1 -v t=$RATIO '{a[NR] = $c} END {
          asort(a);
          d=a[NR]-t;
          d=d < 0 ? -d : d;
          v = a[NR];
          for (i=NR-1;i>=1;i--) {
                  m= a[i] - t;
                  m= m < 0 ? -m : m
                  if (m < d) {
                      d = m;
                      v = a[i];
                  }
          }
          print v
        }')
  FOLDER=$(echo "$RATIOS" | grep "$CLOSEST_RATIO" | awk '{ print $2 }')
  mkdir -p "$1/$FOLDER/"
  mv "$SLOTH_IMAGE_PATH" "$1/$FOLDER/$HASHED_NAME.jpg"
  jpegoptim --size=200k "$1/$FOLDER/$HASHED_NAME.jpg" --overwrite
done
