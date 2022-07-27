function define_element_prop(property, getter) {
  if (!(property in Element.prototype)) {
    Object.defineProperty(Element.prototype, property, {
      get: function() {
        return getter(this);
      }
    });
  }
}

const rootEscape = ["order", "flexGrow", "flexShrink", "flexBasis", "alignSelf", "flex"];

define_element_prop("__stretch_description__", (e) => {
 	var r = describeElement(e);
	if (e.getAttribute("id") === "test-root") {
		if (r.style) {
			for (let key of rootEscape) {
				delete r.style[key];
			}
		}
	}
	return  JSON.stringify(r);
});

function parseDimension(input) {
  if (input.endsWith("px")) {
    return {
      unit: 'points', 
      value: Number(input.replace('px',''))
    };
  } else if (input.endsWith("%")) {
    return {
      unit: 'percent', 
      value: Number(input.replace('%','')) / 100
    };
  } else {
    return input == "auto" ? {unit: "auto"} : undefined;
  }
}

function parseNumber(input) {
  if (input === "" || isNaN(input)) {
    return undefined;
  } else {
    return Number(input);
  }
}

function parseEnum(input) {
  if (input) {
    return input;
  } else {
    return undefined;
  }
}

function parseEdges(edges) {
  var left = parseDimension(edges.left);
  var right = parseDimension(edges.right);
  var top = parseDimension(edges.top);
  var bottom = parseDimension(edges.bottom);
  
  if (left === undefined && right === undefined && top === undefined && bottom === undefined) {
    return undefined;
  }

  return {
    left: left,
    right: right,
    top: top,
    bottom: bottom
  };
}

function parseSize(size) {
  var width = parseDimension(size.width);
  var height = parseDimension(size.height);
  
  if (width === undefined && height === undefined) {
    return undefined;
  }

  return {
    width: width,
    height: height,
  };
}

function describeElement(e) {
  return {
    style: {
      display: parseEnum(e.style.display),

      position_type: parseEnum(e.style.position),
      direction: parseEnum(e.style.direction),
      flexDirection: parseEnum(e.style.flexDirection),

      flexWrap: parseEnum(e.style.flexWrap),
      overflow: parseEnum(e.style.overflow),

      alignItems: parseEnum(e.style.alignItems),
      alignSelf: parseEnum(e.style.alignSelf),
      alignContent: parseEnum(e.style.alignContent),
      
      justifyContent: parseEnum(e.style.justifyContent),

      flexGrow: parseNumber(e.style.flexGrow),
      flexShrink: parseNumber(e.style.flexShrink),
      flexBasis: parseDimension(e.style.flexBasis),

      size: parseSize({width: e.style.width, height: e.style.height}),
      min_size: parseSize({width: e.style.minWidth, height: e.style.minHeight}),
      max_size: parseSize({width: e.style.maxWidth, height: e.style.maxHeight}),

      margin: parseEdges({
        left: e.style.marginLeft,
        right: e.style.marginRight,
        top: e.style.marginTop,
        bottom: e.style.marginBottom,
      }),

      padding: parseEdges({
        left: e.style.paddingLeft,
        right: e.style.paddingRight,
        top: e.style.paddingTop,
        bottom: e.style.paddingBottom,
      }),

      border: parseEdges({
        left: e.style.borderLeftWidth,
        right: e.style.borderRightWidth,
        top: e.style.borderTopWidth,
        bottom: e.style.borderBottomWidth,
      }),

      position: parseEdges({
        left: e.style.left,
        right: e.style.right,
        top: e.style.top,
        bottom: e.style.bottom,
      }),
    },

    layout: {
      width: e.offsetWidth,
      height: e.offsetHeight,
      x: e.offsetLeft,
      y: e.offsetTop,
    },

    children: Array.from(e.children).map(c => describeElement(c)),
  }
}
