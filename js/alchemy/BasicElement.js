Alchemy.ns('Alchemy.v');

/**
 * @class Alchemy.v.BasicElement
 * @extends Alchemy.BaseType
 *
 * The basic type representing a simple GUI element
 */
Alchemy.v.BasicElement = {
    x: undefined,

    y: undefined,

    width: undefined,

    height: undefined,

    constructor: function (cfg) {
        _super.call(this, cfg);
    },

    /**
     * abstract render method
     *
     * @param {Mixed} ctxt
     *      the render context
     *
     * @return {Mixed}
     *      the updated render cntext
     */
    render: function (ctxt) {
        return ctxt;
    },

    update: function () {},

    updateUI: function () {},

    /**
     * moves the element to given position
     *
     * @param {Number} x
     *      the new x-coordinate
     *
     * @param {Number} y
     *      the new y-coordinate
     */
    moveTo: function (x, y) {
        this.x = x;
        this.y = y;
    },

    /**
     * moves the element relative to its previous position
     *
     * @param {Number} dx
     *      the movement at the x-axes
     *
     * @param {Number} dy
     *      the movement at the y-axes
     */
    move: function (dx, dy) {
        this.moveTo(this.getX() + dx, this.getY() + dy);
    },

    /**
     * returns the x-coordinate relative to the elements offset container
     *
     * @return Number
     *      the x-coordinate
     */
    getX: function () {
        return this.x;
    },

    /**
     * returns the y-coordinate relative to the elements offset container
     *
     * @return Number
     *      the y-coordinate
     */
    getY: function () {
        return this.y;
    },

    /**
     * resizes the current element
     *
     * @param {Number} width
     *      the new width
     *
     * @param {Number} height
     *      the new height
     *
     * @return {Boolean}
     *      <code>true</code> iff the element has been resized
     */
    setSize: function (width, height) {
        var widthChanged = this.setWidth(width),
            heightChanged = this.setHeight(height);

        return widthChanged || heightChanged;
    },

    /**
     * changes the width of the current element
     *
     * @param {Number} width
     *      the new width
     *
     * @return {Boolean}
     *      <code>true</code> iff the width has been changed
     */
    setWidth: function (width) {
        if (Alchemy.isNumber(width) && width != this.width) {
            this.width = width;
            return true;
        } else {
            return false;
        }
    },

    /**
     * changes the height of the current element
     *
     * @param {Number} height
     *      the new height
     *
     * @return {Boolean}
     *      <code>true</code> iff the height has been changed
     */
    setHeight: function (height) {
        if (Alchemy.isNumber(height) && height != this.width) {
            this.height = height;
            return true;
        } else {
            return false;
        }
    },

    /**
     * returns the width of the the element
     *
     * @return Number
     *      the width
     */
    getWidth: function () {
        return this.width;
    },

    /**
     * returns the height of the the element
     *
     * @return Number
     *      the height
     */
    getHeight: function () {
        return this.height;
    },

    dispose: function () {
        this.purgeListeners();
        _super.call(this);
    }
};

Alchemy.v.BasicElement = Alchemy.brew({
    extend: Alchemy.util.Observable
}, Alchemy.v.BasicElement);
